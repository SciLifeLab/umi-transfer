//! Parallel processors for 2-file (r1 + UMI) and 3-file (r1 + r2 + UMI) UMI transfer using paraseq.
//! Uses a channel + dedicated writer thread because OutputFile (gzp writer) is not Send.

use anyhow::Result;
use paraseq::fastq::RefRecord;
use paraseq::parallel::{MultiParallelProcessor, PairedParallelProcessor, ProcessError};
use paraseq::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;

use crate::read_editing::{umi_to_record_header, umi_to_record_seq, UMIDestination};
use crate::umi_errors::RuntimeErrors;

fn to_process_error(e: impl std::fmt::Display) -> ProcessError {
    ProcessError::Process(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))
}

/// Convert paraseq FASTQ RefRecord to bio Record for use with read_editing and OutputFile.
/// Splits the header on the first space so id is the sequence id only (matching across r1/r2/umi).
pub fn paraseq_record_to_bio(record: &RefRecord<'_>) -> Result<bio::io::fastq::Record> {
    let id_full = std::str::from_utf8(record.id())
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in record id: {}", e))?;
    let (id_part, desc_part) = match id_full.find(' ') {
        Some(i) => (
            id_full[..i].to_string(),
            Some(id_full[i + 1..].to_string()),
        ),
        None => (
            id_full.to_string(),
            if record.sep().is_empty() {
                None
            } else {
                Some(
                    std::str::from_utf8(record.sep())
                        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in record description: {}", e))?
                        .to_string(),
                )
            },
        ),
    };
    let id = if id_part.starts_with('@') {
        id_part
    } else {
        format!("@{}", id_part)
    };
    let seq = record.seq().into_owned();
    let qual = record
        .qual()
        .map(|q| q.to_vec())
        .unwrap_or_else(Vec::new);
    Ok(bio::io::fastq::Record::with_attrs(
        &id,
        desc_part.as_deref(),
        &seq,
        &qual,
    ))
}

/// Batch message for the writer thread: (batch_id, r1_records, r2_records for 3-file).
pub type BatchMessage = (usize, Vec<bio::io::fastq::Record>, Option<Vec<bio::io::fastq::Record>>);

/// 2-file processor: pairs (r1, ru), transfers UMI to r1, sends r1 batches to writer thread.
#[derive(Clone)]
pub struct UmiPairedProcessor {
    pub tx: Sender<BatchMessage>,
    pub batch_id: Arc<AtomicUsize>,
    pub buffer_r1: Vec<bio::io::fastq::Record>,
    pub target_position: UMIDestination,
    pub edit_nr: bool,
    pub delim: Option<String>,
    pub thread_id: usize,
}

impl UmiPairedProcessor {
    pub fn new(
        tx: Sender<BatchMessage>,
        batch_id: Arc<AtomicUsize>,
        target_position: UMIDestination,
        edit_nr: bool,
        delim: Option<String>,
    ) -> Self {
        Self {
            tx,
            batch_id,
            buffer_r1: Vec::new(),
            target_position,
            edit_nr,
            delim,
            thread_id: 0,
        }
    }
}

impl PairedParallelProcessor<paraseq::fastq::RefRecord<'_>> for UmiPairedProcessor {
    fn process_record_pair(
        &mut self,
        record1: paraseq::fastq::RefRecord<'_>,
        record2: paraseq::fastq::RefRecord<'_>,
    ) -> Result<(), ProcessError> {
        let r1_bio = paraseq_record_to_bio(&record1).map_err(to_process_error)?;
        let ru_bio = paraseq_record_to_bio(&record2).map_err(to_process_error)?;

        if r1_bio.id() != ru_bio.id() {
            return Err(to_process_error(RuntimeErrors::ReadIDMismatch));
        }

        let read_nr = if self.edit_nr { Some(1) } else { None };
        let r1_out = match self.target_position {
            UMIDestination::Header => umi_to_record_header(
                r1_bio,
                ru_bio.seq(),
                self.delim.as_ref(),
                read_nr,
            ),
            UMIDestination::Inline => {
                umi_to_record_seq(r1_bio, ru_bio.seq(), ru_bio.qual(), read_nr)
            }
        }
        .map_err(to_process_error)?;

        self.buffer_r1.push(r1_out);
        Ok(())
    }

    fn on_batch_complete(&mut self) -> Result<(), ProcessError> {
        let id = self.batch_id.fetch_add(1, Ordering::SeqCst);
        let r1 = self.buffer_r1.drain(..).collect();
        self.tx.send((id, r1, None)).map_err(to_process_error)?;
        Ok(())
    }

    fn set_thread_id(&mut self, thread_id: usize) {
        self.thread_id = thread_id;
    }

    fn get_thread_id(&self) -> usize {
        self.thread_id
    }
}

/// 3-file processor: multi (r1, r2, ru), transfers UMI to r1 and r2, sends both to writer thread.
#[derive(Clone)]
pub struct UmiMultiProcessor {
    pub tx: Sender<BatchMessage>,
    pub batch_id: Arc<AtomicUsize>,
    pub buffer_r1: Vec<bio::io::fastq::Record>,
    pub buffer_r2: Vec<bio::io::fastq::Record>,
    pub target_position: UMIDestination,
    pub edit_nr: bool,
    pub delim: Option<String>,
    pub thread_id: usize,
}

impl UmiMultiProcessor {
    pub fn new(
        tx: Sender<BatchMessage>,
        batch_id: Arc<AtomicUsize>,
        target_position: UMIDestination,
        edit_nr: bool,
        delim: Option<String>,
    ) -> Self {
        Self {
            tx,
            batch_id,
            buffer_r1: Vec::new(),
            buffer_r2: Vec::new(),
            target_position,
            edit_nr,
            delim,
            thread_id: 0,
        }
    }
}

impl MultiParallelProcessor<paraseq::fastq::RefRecord<'_>> for UmiMultiProcessor {
    fn process_multi_record(
        &mut self,
        records: &[paraseq::fastq::RefRecord<'_>],
    ) -> Result<(), ProcessError> {
        if records.len() != 3 {
            return Err(ProcessError::MultiRecordMismatch(records.len()));
        }
        let r1_bio = paraseq_record_to_bio(&records[0]).map_err(to_process_error)?;
        let r2_bio = paraseq_record_to_bio(&records[1]).map_err(to_process_error)?;
        let ru_bio = paraseq_record_to_bio(&records[2]).map_err(to_process_error)?;

        if r1_bio.id() != ru_bio.id() || r2_bio.id() != ru_bio.id() {
            return Err(to_process_error(RuntimeErrors::ReadIDMismatch));
        }

        let r1_out = match self.target_position {
            UMIDestination::Header => umi_to_record_header(
                r1_bio,
                ru_bio.seq(),
                self.delim.as_ref(),
                if self.edit_nr { Some(1) } else { None },
            ),
            UMIDestination::Inline => umi_to_record_seq(
                r1_bio,
                ru_bio.seq(),
                ru_bio.qual(),
                if self.edit_nr { Some(1) } else { None },
            ),
        }
        .map_err(to_process_error)?;

        let r2_out = match self.target_position {
            UMIDestination::Header => umi_to_record_header(
                r2_bio,
                ru_bio.seq(),
                self.delim.as_ref(),
                if self.edit_nr { Some(2) } else { None },
            ),
            UMIDestination::Inline => umi_to_record_seq(
                r2_bio,
                ru_bio.seq(),
                ru_bio.qual(),
                if self.edit_nr { Some(2) } else { None },
            ),
        }
        .map_err(to_process_error)?;

        self.buffer_r1.push(r1_out);
        self.buffer_r2.push(r2_out);
        Ok(())
    }

    fn on_batch_complete(&mut self) -> Result<(), ProcessError> {
        let id = self.batch_id.fetch_add(1, Ordering::SeqCst);
        let r1 = self.buffer_r1.drain(..).collect();
        let r2 = self.buffer_r2.drain(..).collect();
        self.tx.send((id, r1, Some(r2))).map_err(to_process_error)?;
        Ok(())
    }

    fn set_thread_id(&mut self, thread_id: usize) {
        self.thread_id = thread_id;
    }

    fn get_thread_id(&self) -> usize {
        self.thread_id
    }
}

/// Run the writer loop: receive batches and return them in a BTreeMap for the main thread to write.
/// The main thread holds OutputFile (not Send) and writes after processing is done.
pub fn collect_batches(
    rx: std::sync::mpsc::Receiver<BatchMessage>,
) -> std::collections::BTreeMap<
    usize,
    (Vec<bio::io::fastq::Record>, Option<Vec<bio::io::fastq::Record>>),
> {
    use std::collections::BTreeMap;
    let mut pending: BTreeMap<
        usize,
        (Vec<bio::io::fastq::Record>, Option<Vec<bio::io::fastq::Record>>),
    > = BTreeMap::new();
    for msg in rx {
        pending.insert(msg.0, (msg.1, msg.2));
    }
    pending
}

/// Write collected batches in order to the output file(s).
/// Records within each batch are sorted by id so output is deterministic across runs.
pub fn write_collected_batches(
    pending: &mut std::collections::BTreeMap<
        usize,
        (Vec<bio::io::fastq::Record>, Option<Vec<bio::io::fastq::Record>>),
    >,
    write_r1: &mut crate::file_io::OutputFile,
    write_r2: &mut Option<crate::file_io::OutputFile>,
) -> anyhow::Result<()> {
    let mut next_id = 0usize;
    while let Some((mut r1, r2_opt)) = pending.remove(&next_id) {
        r1.sort_by(|a, b| a.id().cmp(b.id()));
        for rec in r1 {
            write_r1.write_record(rec)?;
        }
        if let (Some(ref mut w2), Some(mut r2)) = (write_r2.as_mut(), r2_opt) {
            r2.sort_by(|a, b| a.id().cmp(b.id()));
            for rec in r2 {
                w2.write_record(rec)?;
            }
        }
        next_id += 1;
    }
    Ok(())
}
