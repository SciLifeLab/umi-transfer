use clap::Parser;
use file_format::FileFormat;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::iter::Iterator;
use std::thread;

lazy_static::lazy_static! {
static ref UMI_PATTERN: regex::Regex = regex::Regex::new("^(N{2,})([ATCG]*)$").unwrap();
}
// Nucleotide pattern for inline transfer
struct Nucleotide {
    offset: usize,
    spacer: String,
}
// Valid extraction of UMI and read for inline transfer
enum ExtractedRecord {
    Empty,
    Valid {
        read: bio::io::fastq::Record,
        umi: Vec<u8>,
    },
}
// Defining types for simplicity/ readability
type File = std::fs::File;
type Fastq = std::io::BufReader<File>;
type Gzip = flate2::bufread::MultiGzDecoder<Fastq>;

// Enum for the two acceptable input file formats: '.fastq' and '.fastq.gz'
enum ReadFile {
    Fastq(File),
    Gzip(Gzip),
}
impl std::io::Read for ReadFile {
    // Implement read for ReadFile enum
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ReadFile::Fastq(file) => file.read(into),
            ReadFile::Gzip(file) => file.read(into),
        }
    }
}
// Enum for the two accepted output formats, '.fastq' and '.fastq.gz'
enum OutputFile {
    Fastq {
        read: bio::io::fastq::Writer<File>,
    },
    Gzip {
        read: bio::io::fastq::Writer<flate2::write::GzEncoder<File>>,
    },
}
impl OutputFile {
    // Implement write for OutputFile enum
    fn write(self, header: &str, desc: Option<&str>, s: bio::io::fastq::Record) -> OutputFile {
        match self {
            OutputFile::Fastq { mut read } => {
                read.write(header, desc, s.seq(), s.qual()).unwrap();
                OutputFile::Fastq { read }
            }
            OutputFile::Gzip { mut read } => {
                read.write(header, desc, s.seq(), s.qual()).unwrap();
                OutputFile::Gzip { read }
            }
        }
    }
}
// Read input file to Reader. Automatically scans if gzipped from FileFormat crate
fn read_fastq(path: &str) -> bio::io::fastq::Reader<std::io::BufReader<ReadFile>> {
    let format = FileFormat::from_file(path).unwrap();
    if format == FileFormat::Gzip {
        bio::io::fastq::Reader::new(ReadFile::Gzip(
            std::fs::File::open(path)
                .map(std::io::BufReader::new)
                .map(flate2::bufread::MultiGzDecoder::new)
                .unwrap(),
        ))
    } else {
        bio::io::fastq::Reader::new(ReadFile::Fastq(std::fs::File::open(path).unwrap()))
    }
}
//// OLD ver
// // Create output files, gzipped optional
// fn output_file(name: &str, gz: bool) -> OutputFile {
//     if gz {
//         OutputFile::Gzip {
//             read: std::fs::File::create(format!("{}.fastq.gz", name))
//                 .map(|w| flate2::write::GzEncoder::new(w, flate2::Compression::best()))
//                 .map(bio::io::fastq::Writer::new)
//                 .unwrap(),
//         }
//     } else {
//         OutputFile::Fastq {
//             read: std::fs::File::create(format!("{}.fastq", name))
//                 .map(bio::io::fastq::Writer::new)
//                 .unwrap(),
//         }
//     }
// }

// Create output files, gzipped optional
fn output_file(name: &str, gz: bool) -> OutputFile {
    if gz {
        OutputFile::Gzip {
            read: std::fs::File::create(format!("{}.fastq.gz", name))
                .map(|w| flate2::write::GzEncoder::new(w, flate2::Compression::best()))
                .map(bio::io::fastq::Writer::new)
                .unwrap(),
        }
    } else {
        OutputFile::Fastq {
            read: std::fs::File::create(format!("{}.fastq", name))
                .map(bio::io::fastq::Writer::new)
                .unwrap(),
        }
    }
}

#[derive(clap::Parser)]
#[clap(
    version = "0.1.1",
    author = "Judit Hohenthal",
    about = "A tool for transfering Unique Molecular Identifiers (UMIs)."
)]
struct Opts {
    #[clap(
        long,
        default_value = "integrated",
        help = "Prefix for output files, omitted flag will result in default value.
        \n "
    )]
    prefix: String,
    #[clap(
        long,
        required = true,
        help = "[REQUIRED] Input file 1 with reads.
    \n "
    )]
    r1_in: Vec<String>,
    #[clap(
        long,
        help = "Input file 2 with reads.
    \n "
    )]
    r2_in: Vec<String>,
    #[clap(
        long,
        help = "Automatically change '3' into '2' in header of output file from R3.
        \n "
    )]
    edit_nr: bool,
    #[clap(
        long,
        help = "Disable gzipped output file (its enabled by default).
    \n "
    )]
    no_gzip: bool,
    // Subcommands specifying inline or separate extraction
    #[clap(subcommand)]
    sub: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[clap(
        name = "separate",
        about = "If the UMI reads is in separate fastq file 'separate' must be present in command line.
        \nUMI is entered after --ru-in flag.
        \nExample input: 'umi-transfer --no-gzip --r1-in 'example_file.fastq.gz separate --ru-in 'example_umi.fastq.gz''
        \n "
    )]
    Separate {
        #[clap(long, required = true)]
        ru_in: Vec<String>,
    },
    #[clap(
        name = "inline",
        about = "If the UMI appears inline with the input read files 'inline' must be present in command line.
        \n--pattern1 a nucleotide pattern must be available to locate UMI in read file 1
        \n--pattern2 a nucleotide pattern must be available to locate UMI if read file 2 exists
        \nExample input: 'umi-transfer --no-gzip --r1-in 'example_file.fastq' inline --pattern1 'NNNNNNNNN'
        \n "
    )]
    Inline {
        // Patterns for locating UMI inline, given in Nucleotide pattern
        #[clap(long, required = true)]
        pattern1: String,
        #[clap(long)]
        pattern2: Option<String>,
    },
}

// Writes record with properly inserted UMI to Output file
fn write_to_file(
    input: bio::io::fastq::Record,
    output: OutputFile,
    umi: &[u8],
    second: bool,
) -> OutputFile {
    let s = input;
    if second {
        let header = &[s.id(), ":", std::str::from_utf8(&umi).unwrap()].concat();
        let mut string = String::from(s.desc().unwrap());
        string.replace_range(0..1, "2");
        let desc: Option<&str> = Some(&string);
        output.write(header, desc, s)
    } else {
        let header = &[s.id(), ":", std::str::from_utf8(&umi).unwrap()].concat();
        output.write(header, s.desc(), s.clone())
    }
}
// Parses Pattern for Inline extraction
fn parse(pattern: &str) -> Option<Nucleotide> {
    if let Some(captures) = UMI_PATTERN.captures(pattern) {
        Some(Nucleotide {
            offset: captures.get(1)?.end(),
            spacer: captures.get(2)?.as_str().into(),
        })
    } else {
        panic!("")
    }
}
// Extracts UMI from inline record
fn extract(record: bio::io::fastq::Record, pattern: &str) -> ExtractedRecord {
    let handler = parse(pattern);
    match handler {
        Some(Nucleotide { offset, spacer }) => {
            let end = offset + spacer.len();
            if end <= record.seq().len() && record.seq()[offset..end] == *spacer.as_bytes() {
                let read = bio::io::fastq::Record::with_attrs(
                    record.id(),
                    record.desc(),
                    record.seq()[end..record.seq().len()].into(),
                    record.qual()[end..record.qual().len()].into(),
                );
                ExtractedRecord::Valid {
                    read: read,
                    umi: record.seq()[0..offset].into(),
                }
            } else {
                ExtractedRecord::Empty
            }
        }
        None => panic!(""),
    }
}
// Write inline record to Outputfile
fn write_inline_to_file(
    record: ExtractedRecord,
    write_file: OutputFile,
    second: bool,
) -> OutputFile {
    // The record has to have a valid extraxted UMI and read with UMI removed
    match record {
        ExtractedRecord::Empty => panic!("Not Valid UMI/ Record"),
        ExtractedRecord::Valid { read, umi } => write_to_file(read, write_file, &umi, second),
    }
}

fn main() {
    // Parse commandline arguments
    let args = Opts::parse();

    // Automatically gzip output file, if --no-gzip flag was included this will be disabled
    let mut gzip = true;
    if args.no_gzip {
        gzip = false;
    }
    // Create write files, not gzipped if --no-gzip flag entered.
    let mut write_file_r1 = output_file(&format!("{}1", &args.prefix), gzip);

    // Create a record iterator from input file 1
    let r1 = read_fastq(&args.r1_in[0]).records();

    // Settings for progress bar
    let len = read_fastq(&args.r1_in[0]).records().count();
    let m = MultiProgress::new();
    let style = ProgressStyle::with_template("[{elapsed_precise}] {bar:60} {pos:>7}/{len:7} {msg}")
        .unwrap();
    let pb = m.add(ProgressBar::new(len.try_into().unwrap()));
    pb.set_style(style.clone());
    let pb2 = m.insert_after(&pb, ProgressBar::new(len.try_into().unwrap()));
    pb2.set_style(style);
    println!("[1/1] Transfering UMI to records...");

    // Enables editing id in output file 2 if --edit-nr flag was included
    let mut edit_nr = false;
    if args.edit_nr {
        edit_nr = true;
    }
    // Match Subcommand
    match args.sub {
        Commands::Separate { ru_in } => {
            // Clone UMI file for second thread
            let ru1 = ru_in.clone();
            let handle1 = thread::spawn(move || {
                let ru = read_fastq(&ru_in[0]).records();
                // Iterate records in input file and UMI file
                for (r1_rec, ru_rec) in r1.zip(ru) {
                    // Update progress bar
                    pb.set_message("R1");
                    pb.inc(1);
                    // Write to Output file
                    write_file_r1 =
                        write_to_file(r1_rec.unwrap(), write_file_r1, ru_rec.unwrap().seq(), false);
                }
                pb.finish_with_message("R1 done");
            });

            // Save thread handler 1 in Vec
            let mut l = Vec::new();
            l.push(handle1);

            // If input file 2 exists:
            if !&args.r2_in.is_empty() {
                let r2 = read_fastq(&args.r2_in[0]).records();
                let mut write_file_r2 = output_file(&format!("{}2", &args.prefix), gzip);
                let handle2 = thread::spawn(move || {
                    let ru = read_fastq(&ru1[0]).records();

                    // Set progressbar to position 0
                    pb2.set_position(0);
                    for (r2_rec, ru_rec) in r2.zip(ru) {
                        // Update progressbar
                        pb2.set_message("R2");
                        pb2.inc(1);
                        // Write record to Output file
                        write_file_r2 = write_to_file(
                            r2_rec.unwrap(),
                            write_file_r2,
                            ru_rec.unwrap().seq(),
                            edit_nr,
                        );
                    }
                    pb2.finish_with_message("R2 done");
                });
                // Save thread handler 2 in Vec
                l.push(handle2);
            } else {
                // If no recond input file exists, remove second progress bar
                MultiProgress::remove(&m, &pb2);
            }
            // Wait for threads to finish
            for i in l {
                if !i.is_finished() {
                    i.join().unwrap();
                }
            }
        }
        Commands::Inline { pattern1, pattern2 } => {
            // save pattern1 incase its used for both read files
            let mut pat1 = pattern1.clone();
            let handle1 = thread::spawn(move || {
                // Iterate each record in input file 1
                for r1_rec in r1 {
                    // Update progress bar
                    pb.set_message("FASTQ 1");
                    pb.inc(1);

                    // Extract UMI from record and save both
                    let record1 = extract(r1_rec.unwrap(), &pattern1);

                    // Write record and extracted UMI to output file
                    write_file_r1 = write_inline_to_file(record1, write_file_r1, false);
                }
                pb.finish_with_message("FASTQ 1 done");
            });

            // Save thread handler 1 to Vec
            let mut l = Vec::new();
            l.push(handle1);

            if !&args.r2_in.is_empty() {
                // Check if a pattern2 exists
                pat1 = pattern2.unwrap_or_else(|| pat1);
                // create output file
                let mut write_file_r2 = output_file(&format!("{}2", &args.prefix), gzip);
                // create iterator over input file 2
                let r2 = read_fastq(&args.r2_in[0]).records();
                pb2.set_position(0);
                let handle2 = thread::spawn(move || {
                    for r2_rec in r2 {
                        pb2.set_message("FASTQ 2");
                        pb2.inc(1);
                        let record2 = extract(r2_rec.unwrap(), &(pat1));
                        write_file_r2 = write_inline_to_file(record2, write_file_r2, false);
                    }
                    pb2.finish_with_message("FASTQ 2 done");
                });
                l.push(handle2);
            } else {
                MultiProgress::remove(&m, &pb2);
            }
            for i in l {
                if !i.is_finished() {
                    i.join().unwrap();
                }
            }
        }
    }
}
