use anyhow::{anyhow, Context, Result};
use clap::Parser;
use paraseq::fastq;
use paraseq::parallel::{ProcessError, ParallelReader};
use std::path::PathBuf;

use super::file_io;
use crate::auxiliary::{threads_available, threads_per_task};
use crate::paraseq_processor::{
    collect_batches, write_collected_batches, BatchMessage, UmiMultiProcessor, UmiPairedProcessor,
};

#[derive(Debug, Parser)]
pub struct OptsExternal {
    #[clap(
        short = 'p',
        long = "position",
        help = "Choose the target position for the UMI: 'header' or 'inline'. Defaults to 'header'.
        \n ",
        default_value = "header"
    )]
    target_position: crate::read_editing::UMIDestination,
    #[clap(
        short = 'c',
        long = "correct_numbers",
        help = "Read numbers will be altered to ensure the canonical read numbers 1 and 2 in output file sequence headers.
        \n "
    )]
    edit_nr: bool,

    #[clap(
        short = 'z',
        long = "gzip",
        help = "Compress output files. Turned off by default.
        \n "
    )]
    gzip: bool,
    #[clap(
        short = 'l',
        long = "compression_level",
        help = "Choose the compression level: Maximum 9, defaults to 3. Higher numbers result in smaller files but take longer to compress.
        \n "
    )]
    compression_level: Option<u32>,
    #[clap(
        short = 't',
        long = "threads",
        help = "Maximum number of threads to use for processing. Preferably pick odd numbers, 9 or 11 recommended. Defaults to the maximum number of cores available.
        \n "
    )]
    num_threads: Option<usize>,
    #[clap(
        short = 'f',
        long = "force",
        help = "Overwrite existing output files without further warnings or prompts.
        \n "
    )]
    force: bool,
    #[clap(
        short = 'd',
        long = "delim",
        help = "Delimiter to use when joining the UMIs to the read name. Defaults to `:`.
        \n "
    )]
    delim: Option<String>,
    #[clap(
        long = "in",
        required = true,
        help = "[REQUIRED] Input file 1 with reads.
    \n "
    )]
    r1_in: PathBuf,
    #[clap(
        long = "in2",
        help = "Input file 2 with reads (optional). If omitted, only R1 + UMI are processed and a single output is written.
    \n "
    )]
    r2_in: Option<PathBuf>,
    #[clap(
        short = 'u',
        long = "umi",
        required = true,
        help = "[REQUIRED] Input file with UMI.
        \n"
    )]
    ru_in: PathBuf,
    #[clap(
        long = "out",
        help = "Path to FastQ output file for R1.
    \n "
    )]
    r1_out: Option<PathBuf>,
    #[clap(
        long = "out2",
        help = "Path to FastQ output file for R2 (only used when --in2 is provided).
    \n "
    )]
    r2_out: Option<PathBuf>,
}

pub fn run(args: OptsExternal) -> Result<i32> {
    let edit_nr = args.edit_nr;
    let num_threads = args.num_threads.unwrap_or_else(threads_available);

    let num_outputs = if args.r2_in.is_some() { 2 } else { 1 };
    let threads_per_task = threads_per_task(num_threads, num_outputs);

    // Open paraseq readers (handles plain and gzipped via niffler)
    let reader_r1 = fastq::Reader::from_path(&args.r1_in).with_context(|| {
        format!("Failed to open R1 input: {}", args.r1_in.display())
    })?;
    let reader_ru = fastq::Reader::from_path(&args.ru_in).with_context(|| {
        format!("Failed to open UMI input: {}", args.ru_in.display())
    })?;

    if let Some(ref r2_path) = args.r2_in {
        // 3-file mode: r1 + r2 + ru
        let reader_r2 = fastq::Reader::from_path(r2_path)
            .with_context(|| format!("Failed to open R2 input: {}", r2_path.display()))?;

        let mut output1 = args
            .r1_out
            .unwrap_or_else(|| file_io::append_umi_to_path(&args.r1_in));
        let mut output2 = args
            .r2_out
            .unwrap_or_else(|| file_io::append_umi_to_path(r2_path));

        output1 = file_io::rectify_extension(output1, &args.gzip)?;
        output2 = file_io::rectify_extension(output2, &args.gzip)?;
        output1 = file_io::check_outputpath(output1, &args.force)?;
        output2 = file_io::check_outputpath(output2, &args.force)?;

        println!("Output 1 will be saved to: {}", output1.to_string_lossy());
        println!("Output 2 will be saved to: {}", output2.to_string_lossy());

        let mut write_r1 = file_io::create_writer(
            output1,
            &args.gzip,
            &threads_per_task,
            &args.compression_level,
            None,
        )?;
        let write_r2 = file_io::create_writer(
            output2,
            &args.gzip,
            &threads_per_task,
            &args.compression_level,
            None,
        )?;

        let (tx, rx) = std::sync::mpsc::channel::<BatchMessage>();
        let batch_id = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        // Collector thread: only receives batches (Send data) and sends back the map.
        // Main thread keeps OutputFile (not Send) and does the actual writing.
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let collector_handle = std::thread::spawn(move || {
            let _ = result_tx.send(collect_batches(rx));
        });

        let mut processor = UmiMultiProcessor::new(
            tx,
            batch_id,
            args.target_position,
            edit_nr,
            args.delim.clone(),
        );

        println!("Transferring UMIs to records (paired R1+R2)...");
        let process_result = reader_r1.process_parallel_multi(
            vec![reader_r2, reader_ru],
            &mut processor,
            num_threads,
        );
        drop(processor);
        let mut pending = result_rx.recv().unwrap_or_default();
        let _ = collector_handle.join();

        process_result.map_err(process_error_to_anyhow)?;

        let mut write_r2 = Some(write_r2);
        let total_records = write_collected_batches(&mut pending, &mut write_r1, &mut write_r2)?;

        println!("Processed {total_records} records (3-file mode)");
        Ok(0)
    } else {
        // 2-file mode: r1 + ru only
        let mut output1 = args
            .r1_out
            .unwrap_or_else(|| file_io::append_umi_to_path(&args.r1_in));
        output1 = file_io::rectify_extension(output1, &args.gzip)?;
        output1 = file_io::check_outputpath(output1, &args.force)?;

        println!("Output will be saved to: {}", output1.to_string_lossy());

        let mut write_r1 = file_io::create_writer(
            output1,
            &args.gzip,
            &threads_per_task,
            &args.compression_level,
            None,
        )?;

        let (tx, rx) = std::sync::mpsc::channel::<BatchMessage>();
        let batch_id = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));

        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let collector_handle = std::thread::spawn(move || {
            let _ = result_tx.send(collect_batches(rx));
        });

        let mut processor = UmiPairedProcessor::new(
            tx,
            batch_id,
            args.target_position,
            edit_nr,
            args.delim.clone(),
        );

        println!("Transferring UMIs to records (R1 + UMI only)...");
        let process_result = reader_r1
            .process_parallel_paired(reader_ru, &mut processor, num_threads);
        drop(processor);
        let mut pending = result_rx.recv().unwrap_or_default();
        let _ = collector_handle.join();

        process_result.map_err(process_error_to_anyhow)?;

        let mut write_r2 = None;
        let total_records = write_collected_batches(&mut pending, &mut write_r1, &mut write_r2)?;

        println!("Processed {total_records} records (2-file mode)");
        Ok(0)
    }
}

fn process_error_to_anyhow(e: ProcessError) -> anyhow::Error {
    anyhow!("{}", e)
}
