use anyhow::{anyhow, Context, Result};
use clap::Parser;
use itertools::izip;
use std::path::PathBuf;

use super::file_io;
use crate::{auxiliary::check_outputpath, umi_errors::RuntimeErrors};
#[derive(Debug, Parser)]
pub struct OptsExternal {
    #[clap(
        short = 'f',
        long = "fix_numbers",
        help = "Automatically change '3' into '2' in sequence header of output file from R3.
        \n "
    )]
    edit_nr: bool,
    #[clap(
        short = 'z',
        long = "gzip",
        help = "Compress output files with gzip. By default turned off to encourage use of external compression (see Readme).
        \n "
    )]
    gzip: bool,
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
        long,
        default_value = "output",
        help = "Prefix for output files, omitted flag will result in default value.
        \n "
    )]
    prefix: String,
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

    // Create fastq record iterators from input files
    let r1 = file_io::read_fastq(&args.r1_in).records();
    let r2 = file_io::read_fastq(&args.r2_in).records();
    let ru = file_io::read_fastq(&args.ru_in).records();

    // If output paths have been specified, check if the are ok to use or use prefix constructors.
    let output1: PathBuf;
    let output2: PathBuf;

    if args.r1_out.is_some() && args.r2_out.is_some() {
        output1 = check_outputpath(args.r1_out.unwrap())?;
        output2 = check_outputpath(args.r2_out.unwrap())?;
    } else {
        output1 = check_outputpath(PathBuf::from(format!("{}1", &args.prefix)))?;
        output2 = check_outputpath(PathBuf::from(format!("{}2", &args.prefix)))?;
    }

    let mut write_file_r1 = file_io::output_file(output1, args.gzip);
    let mut write_file_r2 = file_io::output_file(output2, args.gzip);

    // Record counter
    let mut counter: i32 = 0;

    println!("Transferring UMIs to records...");

    // Iterate over records in input files
    for (r1_rec_res, ru_rec_res, r2_rec_res) in izip!(r1, ru, r2) {
        let r1_rec = r1_rec_res.with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.r1_in.to_string_lossy()
            )
        })?;
        let r2_rec = r2_rec_res.with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.r2_in.to_string_lossy()
            )
        })?;
        let ru_rec = ru_rec_res.with_context(|| {
            format!(
                "Failed to read records from {}",
                &args.ru_in.to_string_lossy()
            )
        })?;

        // Step counter
        counter += 1;

        if r1_rec.id().eq(ru_rec.id()) {
            // Write to Output file (never edit nr for R1)
            write_file_r1 = file_io::write_to_file(r1_rec, write_file_r1, &ru_rec.seq(), false);
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatchError));
        }

        if r2_rec.id().eq(ru_rec.id()) {
            // Write to Output file (edit nr for R2 if --edit-nr flag was included)
            write_file_r2 = file_io::write_to_file(r2_rec, write_file_r2, &ru_rec.seq(), edit_nr);
        } else {
            return Err(anyhow!(RuntimeErrors::ReadIDMismatchError));
        }
    }
    println!("Processed {:?} records", counter);
    Ok(counter)
}
