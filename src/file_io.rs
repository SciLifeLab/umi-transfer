use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use file_format::FileFormat;
use std::{fs, path::PathBuf};

use super::umi_errors::RuntimeErrors;

// Defining types for simplicity
type File = std::fs::File;
type Fastq = std::io::BufReader<File>;
type Gzip = flate2::bufread::MultiGzDecoder<Fastq>;

// Enum for the two acceptable input file formats: '.fastq' and '.fastq.gz'
pub enum ReadFile {
    Fastq(File),
    Gzip(Gzip),
}

// Implement read for ReadFile enum
impl std::io::Read for ReadFile {
    fn read(&mut self, into: &mut [u8]) -> std::io::Result<usize> {
        match self {
            ReadFile::Fastq(file) => file.read(into),
            ReadFile::Gzip(file) => file.read(into),
        }
    }
}

// Enum for the two accepted output formats, '.fastq' and '.fastq.gz'
pub enum OutputFile {
    Fastq {
        read: bio::io::fastq::Writer<File>,
    },
    Gzip {
        read: bio::io::fastq::Writer<flate2::write::GzEncoder<File>>,
    },
}

// Implement write for OutputFile enum
impl OutputFile {
    pub fn write(self, header: &str, desc: Option<&str>, s: bio::io::fastq::Record) -> OutputFile {
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

// Read input file to Reader. Automatically scans if gzipped from file-format crate
pub fn read_fastq(path: &PathBuf) -> bio::io::fastq::Reader<std::io::BufReader<ReadFile>> {
    let format = FileFormat::from_file(path).unwrap();
    if format == FileFormat::Gzip {
        bio::io::fastq::Reader::new(ReadFile::Gzip(
            std::fs::File::open(path)
                .map(std::io::BufReader::new)
                .map(flate2::bufread::MultiGzDecoder::new)
                .unwrap(),
        ))
    } else {
        // If not gzipped, read as plain fastq
        bio::io::fastq::Reader::new(ReadFile::Fastq(std::fs::File::open(path).unwrap()))
    }
}

// Create output files
pub fn output_file(name: PathBuf) -> OutputFile {
    if name.ends_with(".gz") {
        OutputFile::Gzip {
            read: std::fs::File::create(name.as_path())
                .map(|w| flate2::write::GzEncoder::new(w, flate2::Compression::default()))
                .map(bio::io::fastq::Writer::new)
                .unwrap(),
        }
    } else {
        OutputFile::Fastq {
            read: std::fs::File::create(name.as_path())
                .map(bio::io::fastq::Writer::new)
                .unwrap(),
        }
    }
}

// Writes record with properly inserted UMI to Output file
pub fn write_to_file(
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

// Checks whether an output path exists.
pub fn check_outputpath(mut path: PathBuf, compress: &bool) -> Result<PathBuf> {
    // handle the compression and adapt file extension.
    if compress | path.ends_with(".gz") {
        path.set_extension("fastq.gz");
    } else {
        path.set_extension("fastq");
    }

    // check if the path already exists
    let exists = fs::metadata(&path).is_ok();

    // return the path of it is ok to write, otherwise an error.
    if exists {
        if Confirm::new()
            .with_prompt(format!("{} exists. Overwrite?", path.display()))
            .interact()?
        {
            println!("File will be overwritten.");
            return Ok(path);
        } else {
            return Err(anyhow!(RuntimeErrors::FileExistsError));
        }
    } else {
        return Ok(path);
    }
}

// probably, there is a better way to do this than with two copies ?!?
pub fn append_to_path(path: &PathBuf, string: &str) -> PathBuf {
    let mut stem = path.to_owned();
    stem.set_extension("");
    let mut p_osstr = stem.as_os_str().to_owned();
    p_osstr.push(string);
    p_osstr.into()
}
