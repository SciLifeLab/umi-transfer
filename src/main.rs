extern crate core;

use anyhow::{Context, Result};
use clap::Parser;

use crate::auxiliary::timedrun;
use crate::umi_external::OptsExternal;
///use crate::umi_internal::OptsInternal;
mod auxiliary;
mod file_io;
mod umi_errors;
mod umi_external;

#[derive(clap::Parser)]
#[clap(
    version = "0.2.0",
    author = "Written by Judit Hohenthal, Matthias Zepper, Johannes Alneberg",
    about = "A tool for transferring Unique Molecular Identifiers (UMIs). \n\nThe UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the header of the first two fastq files. \n\n"
)]

pub struct Opt {
    #[clap(subcommand)]
    cmd: Subcommand,
}

#[derive(Debug, Parser)]
enum Subcommand {
    /// Integrate UMIs from a separate FastQ file.
    External(OptsExternal),
    // Extract UMIs from the reads themselves.
    // Internal(OptsInternal),
}

fn main() {
    let opt: Opt = Opt::parse();
    timedrun("umi-transfer finished ", || {
        let res = match opt.cmd {
            Subcommand::External(arg) => {
                umi_external::run(arg).context("Failed to include the UMIs")
            } //Subcommand::Internal(arg) => umi_internal::run(arg),
        };

        if let Err(v) = res {
            println!("{:?}", v)
        }
    });
}
