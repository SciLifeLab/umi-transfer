use anyhow::{anyhow, Context, Result};
use clap::Parser;
use itertools::izip;
use std::path::PathBuf;

use super::file_io;
use crate::auxiliary::{threads_available, threads_per_task};
use crate::umi_errors::RuntimeErrors;
#[derive(Debug, Parser)]
pub struct OptsExternal {
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
        help = "Number of threads to use for processing. Defaults to the number of logical cores available.
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
    let num_threads = args.num_threads.unwrap_or_else(|| threads_available());

    // Determine the number of threads available for output file compression.
    let threads_per_task = threads_per_task(num_threads, 2);

    // Read FastQ records from input files
    let r1 = file_io::read_fastq(&args.r1_in)
        .with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.r1_in.to_string_lossy()
            )
        })?
        .records();
    let r2 = file_io::read_fastq(&args.r2_in)
        .with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.r2_in.to_string_lossy()
            )
        })?
        .records();
    let ru = file_io::read_fastq(&args.ru_in)
        .with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.ru_in.to_string_lossy()
            )
        })?
        .records();

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

    // Iterate over records in input files
    for (r1_rec_res, ru_rec_res, r2_rec_res) in izip!(r1, ru, r2) {
        let r1_rec = r1_rec_res?;
        let r2_rec = r2_rec_res?;
        let ru_rec = ru_rec_res?;

        // Step counter
        counter += 1;

        if r1_rec.id().eq(ru_rec.id()) {
            // Write to Output file
            let read_nr = if edit_nr { Some(1) } else { None };
            let r1_rec = update_record(r1_rec, ru_rec.seq(), args.delim.as_ref(), read_nr)?;

            write_output_r1.write_record(r1_rec)?;
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatch));
        }

        if r2_rec.id().eq(ru_rec.id()) {
            // Write to Output file
            let read_nr = if edit_nr { Some(2) } else { None };
            let r2_rec = update_record(r2_rec, ru_rec.seq(), args.delim.as_ref(), read_nr)?;

            write_output_r2.write_record(r2_rec)?;
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatch));
        }
    }
    println!("Processed {:?} records", counter);
    Ok(counter)
}

// Updates the header and description of the reads accordingly
fn update_record(
    input: bio::io::fastq::Record,
    umi: &[u8],
    umi_sep: Option<&String>,
    edit_nr: Option<u8>,
) -> Result<bio::io::fastq::Record> {
    let delim = umi_sep.as_ref().map(|s| s.as_str()).unwrap_or(":"); // the delimiter for the UMI
    if let Some(number) = edit_nr {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let mut new_desc = String::from(input.desc().unwrap());
        new_desc.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&new_desc);
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, desc, input.seq(), input.qual());
        Ok(new_record)
    } else {
        let new_id = &[input.id(), delim, std::str::from_utf8(umi).unwrap()].concat();
        let new_record =
            bio::io::fastq::Record::with_attrs(new_id, input.desc(), input.seq(), input.qual());
        Ok(new_record)
    }
}
