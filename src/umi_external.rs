use anyhow::{anyhow, Context, Result};
use clap::Parser;
use itertools::izip;
use std::path::PathBuf;

use super::file_io;
use crate::{file_io::check_outputpath, umi_errors::RuntimeErrors};
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
        help = "Compress output files. By default, turned off in favour of external compression.
        \n "
    )]
    gzip: bool,
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

    // modify if output path according to compression settings and check if exists.
    output1 = check_outputpath(output1, &args.gzip, &args.force)?;
    output2 = check_outputpath(output2, &args.gzip, &args.force)?;

    println!("Output 1 will be saved to: {}", output1.to_string_lossy());
    println!("Output 2 will be saved to: {}", output2.to_string_lossy());

    let mut write_file_r1 = file_io::output_file(output1);
    let mut write_file_r2 = file_io::output_file(output2);

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
            write_file_r1 = file_io::write_to_file(
                r1_rec,
                write_file_r1,
                ru_rec.seq(),
                args.delim.as_ref(),
                read_nr,
            );
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatchError));
        }

        if r2_rec.id().eq(ru_rec.id()) {
            // Write to Output file
            let read_nr = if edit_nr { Some(2) } else { None };
            write_file_r2 = file_io::write_to_file(
                r2_rec,
                write_file_r2,
                ru_rec.seq(),
                args.delim.as_ref(),
                read_nr,
            );
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatchError));
        }
    }
    println!("Processed {:?} records", counter);
    Ok(counter)
}
