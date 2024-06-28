extern crate core;

use anyhow::Context;
use clap::Parser;
use owo_colors::{OwoColorize, Stream::Stderr, Stream::Stdout};

use std::process;

use crate::auxiliary::timedrun;
use crate::umi_external::OptsExternal;
///use crate::umi_internal::OptsInternal;
mod auxiliary;
mod file_io;
mod umi_errors;
mod umi_external;

const LOGO: &str = r#"
░░░░░░░░░░░░░░░░░░░░░░░░░░░ SciLifeLab - National Genomics Infrastructure ░░░░░░░░░░░░░░░░░░░░░░░░░░░                                              
"#;

const WEB: &str = r#"https://www.scilifelab.se
https://ngisweden.scilifelab.se
https://github.com/SciLifeLab/umi-transfer                                      
"#;

#[derive(clap::Parser)]
#[clap(
    version = "1.5.0",
    author = "Written by Matthias Zepper, Judit Hohenthal & Johannes Alneberg",
    about = "A tool for transferring Unique Molecular Identifiers (UMIs).",
    long_about = "Most tools capable of using UMIs to increase the accuracy of quantitative DNA sequencing experiments expect the respective UMI sequence to be embedded into the reads' IDs. You can use `umi-transfer external` to retrieve UMIs from a separate FastQ file and embed them to the IDs of your paired FastQ files."
)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
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
    println!(
        "\n{}",
        LOGO.if_supports_color(Stdout, |text| text.fg_rgb::<0xA7, 0xC9, 0x47>())
    );
    //println!("{}", WEB.fg_rgb::<0x49, 0x1F, 0x53>().italic());
    println!(
        "{}",
        WEB.if_supports_color(Stdout, |text| text.fg_rgb::<0x6F, 0x6F, 0x6F>())
    );

    // for custom styles of clap parsing errors and help message
    let opt: Opt = Opt::try_parse().unwrap_or_else(|err| {
        match err.kind() {
            // rust render as Clap would by default.
            clap::error::ErrorKind::DisplayHelp
            | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | clap::error::ErrorKind::DisplayVersion => {
                err.print().unwrap();
                process::exit(0);
            }
            _ => {
                // color green for consistency with Runtime errors. Sadly no styled formatting yet.
                eprintln!(
                    "{}",
                    err.render()
                        .if_supports_color(Stderr, |text| text.fg_rgb::<0xA7, 0xC9, 0x47>())
                );
                process::exit(1);
            }
        };
    });

    timedrun("umi-transfer finished", || {
        let res = match opt.cmd {
            Subcommand::External(arg) => {
                umi_external::run(arg).context("Failed to include the UMIs")
            } //Subcommand::Internal(arg) => umi_internal::run(arg),
        };

        if let Err(err) = res {
            eprintln!(
                "{:?}",
                err.if_supports_color(Stderr, |text| text.fg_rgb::<0xA7, 0xC9, 0x47>())
            );
            process::exit(1);
        }
    });
}
