use anyhow::{Context, Result};
use itertools::izip;

use super::file_io;

pub fn run(args: super::Opts) -> Result<()> {
    // Enables editing id in output file 2 if --edit-nr flag was included
    let mut edit_nr = false;
    if args.edit_nr {
        edit_nr = true;
    }

    // Create fastq record iterators from input files
    let r1 = file_io::read_fastq(&args.r1_in[0]).records();
    let r2 = file_io::read_fastq(&args.r2_in[0]).records();
    let ru = file_io::read_fastq(&args.ru_in[0]).records();

    // Create write files.
    let mut write_file_r1 = file_io::output_file(&format!("{}1", &args.prefix), args.gzip);
    let mut write_file_r2 = file_io::output_file(&format!("{}2", &args.prefix), args.gzip);

    println!("Transferring UMIs to records...");

    // Iterate over records in input files
    for (r1_rec, ru_rec_res, r2_rec) in izip!(r1, ru, r2) {
        let ru_rec = ru_rec_res
            .with_context(|| format!("Failed to read records from {}", &args.ru_in[0]))?;

        // Write to Output file (never edit nr for R1)
        write_file_r1 = file_io::write_to_file(
            r1_rec.with_context(|| format!("Failed to read records from {}", &args.r1_in[0]))?,
            write_file_r1,
            &ru_rec.seq(),
            false,
        );

        // Write to Output file (edit nr for R2 if --edit-nr flag was included)
        write_file_r2 = file_io::write_to_file(
            r2_rec.with_context(|| format!("Failed to read records from {}", &args.r2_in[0]))?,
            write_file_r2,
            &ru_rec.seq(),
            edit_nr,
        );
    }
    Ok(())
}
