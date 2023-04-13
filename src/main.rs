use clap::Parser;
use itertools::izip;
use std::iter::Iterator;

// Defining types for simplicity
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
    Fastq { read: bio::io::fastq::Writer<File> },
}

impl OutputFile {
    // Implement write for OutputFile enum
    fn write(self, header: &str, desc: Option<&str>, s: bio::io::fastq::Record) -> OutputFile {
        match self {
            OutputFile::Fastq { mut read } => {
                read.write(header, desc, s.seq(), s.qual()).unwrap();
                OutputFile::Fastq { read }
            }
        }
    }
}

// Read input file to Reader. Automatically scans if gzipped from .gz suffix
fn read_fastq(path: &str) -> bio::io::fastq::Reader<std::io::BufReader<ReadFile>> {
    if path.ends_with(".gz") {
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

// Create output files
fn output_file(name: &str) -> OutputFile {
    OutputFile::Fastq {
        read: std::fs::File::create(format!("{}.fastq", name))
            .map(bio::io::fastq::Writer::new)
            .unwrap(),
    }
}

#[derive(clap::Parser)]
#[clap(
    version = "0.2.0",
    author = "Judit Hohenthal, Matthias Zepper, Johannes Alneberg",
    about = "A tool for transfering Unique Molecular Identifiers (UMIs). \n\nThe UMIs are given as a fastq file and will be transferred, explaining the name umi-transfer, to the header of the first two fastq files. \n\n"
)]
struct Opts {
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
}

// Writes record with properly inserted UMI to Output file
fn write_to_file(
    input: bio::io::fastq::Record,
    output: OutputFile,
    umi: &[u8],
    edit_nr: bool,
) -> OutputFile {
    let s = input;
    if edit_nr {
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

fn main() {
    // Parse commandline arguments
    let args = Opts::parse();

    // Enables editing id in output file 2 if --edit-nr flag was included
    let mut edit_nr = false;
    if args.edit_nr {
        edit_nr = true;
    }

    // Create fastq record iterators from input files
    let r1 = read_fastq(&args.r1_in[0]).records();
    let r2 = read_fastq(&args.r2_in[0]).records();
    let ru = read_fastq(&args.ru_in[0]).records();

    // Create write files.
    let mut write_file_r1 = output_file(&format!("{}1", &args.prefix));
    let mut write_file_r2 = output_file(&format!("{}2", &args.prefix));

    println!("Transfering UMIs to records...");

    // Iterate over records in input files
    for (r1_rec, ru_rec_res, r2_rec) in izip!(r1, ru, r2) {
        let ru_rec = ru_rec_res.unwrap();
        // Write to Output file (never edit nr for R1)
        write_file_r1 = write_to_file(r1_rec.unwrap(), write_file_r1, ru_rec.seq(), false);

        let ru_rec2 = ru_rec.clone();
        // Write to Output file (edit nr for R2 if --edit-nr flag was included)
        write_file_r2 = write_to_file(r2_rec.unwrap(), write_file_r2, ru_rec2.seq(), edit_nr);
    }
}
