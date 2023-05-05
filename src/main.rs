use clap::Parser;

mod file_io;
mod umi_external;

#[derive(clap::Parser)]
#[clap(
    version = "0.2.0",
    author = "Written by Judit Hohenthal, Matthias Zepper, Johannes Alneberg",
    about = "A tool for transfering Unique Molecular Identifiers (UMIs). \n\nThe UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the header of the first two fastq files. \n\n"
)]
pub struct Opts {
    #[clap(
        long,
        default_value = "output",
        help = "Prefix for output files, omitted flag will result in default value.
        \n "
    )]
    prefix: String,
    #[clap(
        long,
        help = "Automatically change '3' into '2' in sequence header of output file from R3.
        \n "
    )]
    edit_nr: bool,
    #[clap(
        long,
        required = true,
        help = "[REQUIRED] Input file 1 with reads.
    \n "
    )]
    r1_in: Vec<String>,
    #[clap(
        long,
        required = true,
        help = "[REQUIRED] Input file 2 with reads.
    \n "
    )]
    r2_in: Vec<String>,
    #[clap(
        long,
        required = true,
        help = "[REQUIRED] Input file with UMI.
        \n"
    )]
    ru_in: Vec<String>,
    #[clap(
        long,
        help = "Compress output files with gzip. By default turned off to encourage use of external compression (see Readme).
        \n "
    )]
    gzip: bool,
}

fn main() {
    // Parse commandline arguments
    let args = Opts::parse();

    umi_external::run(args);
}
