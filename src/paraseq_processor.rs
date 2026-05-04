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
use crate::record::OwnedRecord;
use crate::umi_errors::RuntimeErrors;

fn to_process_error(e: impl std::fmt::Display) -> ProcessError {
    ProcessError::Process(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        e.to_string(),
    )))
}

/// Convert a paraseq FASTQ [`RefRecord`] to an [`OwnedRecord`].
///
/// The `head` field receives everything on the header line after the `@`
/// (id + optional space + description), exactly as paraseq exposes it via `id()`.
fn paraseq_record_to_owned(record: &RefRecord<'_>) -> OwnedRecord {
    OwnedRecord::new(
        record.id().to_vec(),
        record.seq().into_owned(),
        record.qual().unwrap_or(&[]).to_vec(),
    )
}

/// Batch message for the writer thread: `(batch_id, r1_records, r2_records_for_3_file)`.
pub type BatchMessage = (usize, Vec<OwnedRecord>, Option<Vec<OwnedRecord>>);

/// 2-file processor: pairs (r1, ru), transfers UMI to r1, sends r1 batches to writer thread.
#[derive(Clone)]
pub struct UmiPairedProcessor {
    pub tx: Sender<BatchMessage>,
    pub batch_id: Arc<AtomicUsize>,
    pub buffer_r1: Vec<OwnedRecord>,
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
        let r1 = paraseq_record_to_owned(&record1);
        let ru = paraseq_record_to_owned(&record2);

        if r1.id() != ru.id() {
            return Err(to_process_error(RuntimeErrors::ReadIDMismatch));
        }

        let read_nr = if self.edit_nr { Some(1) } else { None };
        let r1_out = match self.target_position {
            UMIDestination::Header => {
                umi_to_record_header(r1, &ru.seq, self.delim.as_ref(), read_nr)
            }
            UMIDestination::Inline => umi_to_record_seq(r1, &ru.seq, &ru.qual, read_nr),
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
    pub buffer_r1: Vec<OwnedRecord>,
    pub buffer_r2: Vec<OwnedRecord>,
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
        let r1 = paraseq_record_to_owned(&records[0]);
        let r2 = paraseq_record_to_owned(&records[1]);
        let ru = paraseq_record_to_owned(&records[2]);

        if r1.id() != ru.id() || r2.id() != ru.id() {
            return Err(to_process_error(RuntimeErrors::ReadIDMismatch));
        }

        let r1_out = match self.target_position {
            UMIDestination::Header => umi_to_record_header(
                r1,
                &ru.seq,
                self.delim.as_ref(),
                if self.edit_nr { Some(1) } else { None },
            ),
            UMIDestination::Inline => umi_to_record_seq(
                r1,
                &ru.seq,
                &ru.qual,
                if self.edit_nr { Some(1) } else { None },
            ),
        }
        .map_err(to_process_error)?;

        let r2_out = match self.target_position {
            UMIDestination::Header => umi_to_record_header(
                r2,
                &ru.seq,
                self.delim.as_ref(),
                if self.edit_nr { Some(2) } else { None },
            ),
            UMIDestination::Inline => umi_to_record_seq(
                r2,
                &ru.seq,
                &ru.qual,
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

/// Receive batch messages from the channel and write them to the output file(s) in order,
/// maintaining a small pending queue for out-of-order arrivals.
///
/// Because each batch is written and freed as soon as it is the next in sequence, memory
/// usage is bounded by `paraseq_threads × batch_size` records rather than the total file
/// size.  At most `paraseq_threads` batches sit in `pending` simultaneously.
pub fn stream_write_batches(
    rx: std::sync::mpsc::Receiver<BatchMessage>,
    write_r1: &mut crate::file_io::OutputFile,
    write_r2: &mut Option<crate::file_io::OutputFile>,
) -> anyhow::Result<usize> {
    use std::collections::BTreeMap;
    let mut pending: BTreeMap<usize, (Vec<OwnedRecord>, Option<Vec<OwnedRecord>>)> =
        BTreeMap::new();
    let mut next_id: usize = 0;
    let mut total_records = 0usize;

    for (id, r1, r2_opt) in rx {
        pending.insert(id, (r1, r2_opt));

        // Drain every consecutive batch that is now ready to write.
        while let Some((mut r1, r2_opt)) = pending.remove(&next_id) {
            r1.sort_by(|a, b| a.id().cmp(b.id()));
            total_records += r1.len();
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
    }
    Ok(total_records)
}
