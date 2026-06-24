use anyhow::{anyhow, Context, Result};
use clap::Parser;
use seq_io::fastq::Record;
use std::path::PathBuf;

use super::file_io;
use crate::auxiliary::{threads_available, threads_per_task};
use crate::read_editing::{read_id, write_umi_in_header, write_umi_inline, UMIDestination};
use crate::umi_errors::RuntimeErrors;
#[derive(Debug, Parser)]
pub struct OptsExternal {
    #[clap(
        short = 'p',
        long = "position",
        help = "Choose the target position for the UMI: 'header' or 'inline'. Defaults to 'header'.
        \n ",
        default_value = "header"
    )]
    target_position: UMIDestination,
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
    //#[clap(
    //    short = 'p',
    //    long = "pin_threads",
    //    help = "Pin threads to physical cores. This can provide a significant performance improvement, but has the downside of possibly conflicting with other pinned cores.
    //    \n "
    //)]
    // pin_threads: bool,
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
        required = true,
        help = "[REQUIRED] Input file 2 with reads.
    \n "
    )]
    r2_in: PathBuf,
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
        help = "Path to FastQ output file for R2.
    \n "
    )]
    r2_out: Option<PathBuf>,
}

pub fn run(args: OptsExternal) -> Result<i32> {
    // Enables editing id in output file 2 if --edit-nr flag was included
    let mut edit_nr = false;
    if args.edit_nr {
        edit_nr = true;
    }

    // Set the number of threads to max, unless manually specified. In case of failure, use only 1.
    let num_threads = args.num_threads.unwrap_or_else(threads_available);

    // Determine the number of threads available for output file compression.
    let threads_per_task = threads_per_task(num_threads, 2);

    // Read FastQ records from input files
    let mut r1 = file_io::read_fastq(&args.r1_in).with_context(|| {
        format!(
            "Failed to read records from {}",
            &args.r1_in.to_string_lossy()
        )
    })?;
    let mut r2 = file_io::read_fastq(&args.r2_in).with_context(|| {
        format!(
            "Failed to read records from {}",
            &args.r2_in.to_string_lossy()
        )
    })?;
    let mut ru = file_io::read_fastq(&args.ru_in).with_context(|| {
        format!(
            "Failed to read records from {}",
            &args.ru_in.to_string_lossy()
        )
    })?;

    // If output paths have been specified, check if the are ok to use or use prefix constructors.
    let mut output1: PathBuf = args
        .r1_out
        .unwrap_or(file_io::append_umi_to_path(&args.r1_in));
    let mut output2: PathBuf = args
        .r2_out
        .unwrap_or(file_io::append_umi_to_path(&args.r2_in));

    // set the correct extension.
    output1 = file_io::rectify_extension(output1, &args.gzip)?;
    output2 = file_io::rectify_extension(output2, &args.gzip)?;

    // modify if output path according to compression settings and check if exists.
    output1 = file_io::check_outputpath(output1, &args.force)?;
    output2 = file_io::check_outputpath(output2, &args.force)?;

    println!("Output 1 will be saved to: {}", output1.to_string_lossy());
    println!("Output 2 will be saved to: {}", output2.to_string_lossy());

    let mut write_output_r1 = file_io::create_writer(
        output1,
        &args.gzip,
        &threads_per_task,
        &args.compression_level,
        None,
    )?;
    let mut write_output_r2 = file_io::create_writer(
        output2,
        &args.gzip,
        &threads_per_task,
        &args.compression_level,
        None,
    )?;

    // Record counter
    let mut counter: i32 = 0;

    println!("Transferring UMIs to records...");

    // Resolve the UMI delimiter once (defaults to ":").
    let delim = args.delim.as_deref().unwrap_or(":").as_bytes();
    let nr1 = if edit_nr { Some(1) } else { None };
    let nr2 = if edit_nr { Some(2) } else { None };

    // Iterate over the three inputs in lockstep. seq_io lends each record as a
    // borrow into its reader's buffer, so we splice and write it within the same
    // iteration, before the next read overwrites the buffer. No record is owned.
    loop {
        let (rec1, rec2, recu) = match (r1.next(), r2.next(), ru.next()) {
            (Some(rec1), Some(rec2), Some(recu)) => (rec1?, rec2?, recu?),
            (None, None, None) => break,
            _ => return Err(anyhow!(RuntimeErrors::RecordCountMismatch)),
        };

        counter += 1;

        // The read and its UMI must be the same record. seq_io has already
        // validated the FASTQ framing; this checks the ids line up.
        let umi_id = read_id(recu.head());
        if read_id(rec1.head()) != umi_id || read_id(rec2.head()) != umi_id {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatch));
        }

        match args.target_position {
            UMIDestination::Header => {
                write_umi_in_header(
                    &mut write_output_r1,
                    rec1.head(),
                    rec1.seq(),
                    rec1.qual(),
                    recu.seq(),
                    delim,
                    nr1,
                )?;
                write_umi_in_header(
                    &mut write_output_r2,
                    rec2.head(),
                    rec2.seq(),
                    rec2.qual(),
                    recu.seq(),
                    delim,
                    nr2,
                )?;
            }
            UMIDestination::Inline => {
                write_umi_inline(
                    &mut write_output_r1,
                    rec1.head(),
                    rec1.seq(),
                    rec1.qual(),
                    recu.seq(),
                    recu.qual(),
                    nr1,
                )?;
                write_umi_inline(
                    &mut write_output_r2,
                    rec2.head(),
                    rec2.seq(),
                    rec2.qual(),
                    recu.seq(),
                    recu.qual(),
                    nr2,
                )?;
            }
        }
    }

    // Flush and finalize (gzip footer) before reporting success.
    write_output_r1.finish()?;
    write_output_r2.finish()?;

    println!("Processed {:?} records", counter);
    Ok(counter)
}
