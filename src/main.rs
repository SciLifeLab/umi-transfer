extern crate core;

use anyhow::Context;
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
    author = "Written by Judit Hohenthal, Matthias Zepper & Johannes Alneberg",
    about = "A tool for transferring Unique Molecular Identifiers (UMIs).\n\nMost tools capable of using UMIs to increase the accuracy of quantitative DNA sequencing experiments expect the respective UMI sequence to be embedded into the reads' IDs.\n\n You can use `umi-transfer external` to retrieve UMIs from a separate FastQ file and embed them to the IDs of your paired FastQ files.\n\n"
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
    timedrun("umi-transfer finished", || {
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
