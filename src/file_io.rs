use super::umi_errors::RuntimeErrors;
use anyhow::{anyhow, Context, Result};
use dialoguer::Confirm;
use file_format::FileFormat;
use regex::Regex;
use std::{fs, path::PathBuf};

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

// Read input file to Reader. Automatically scans if input is compressed with file-format crate.
pub fn read_fastq(path: &PathBuf) -> Result<bio::io::fastq::Reader<std::io::BufReader<ReadFile>>> {
    if fs::metadata(&path).is_err() {
        return Err(anyhow!(RuntimeErrors::FileNotFoundError));
    }

    let format = FileFormat::from_file(path).context("Failed to determine file format")?;
    if format == FileFormat::Gzip {
        Ok(bio::io::fastq::Reader::new(ReadFile::Gzip(
            std::fs::File::open(path)
                .map(std::io::BufReader::new)
                .map(flate2::bufread::MultiGzDecoder::new)
                .with_context(|| format!("Failed to open file: {:?}", path))?,
        )))
    } else {
        // If not gzipped, read as plain fastq
        Ok(bio::io::fastq::Reader::new(ReadFile::Fastq(
            std::fs::File::open(path)
                .with_context(|| format!("Failed to open file: {:?}", path))?,
        )))
    }
}

// Create output files
pub fn output_file(name: PathBuf) -> OutputFile {
    if let Some(extension) = name.extension() {
        if extension == "gz" {
            // File has gz extension, which has been enforced by check_outputpath() if -z was provided.
            OutputFile::Gzip {
                read: std::fs::File::create(name.as_path())
                    .map(|w| flate2::write::GzEncoder::new(w, flate2::Compression::default()))
                    .map(bio::io::fastq::Writer::new)
                    .unwrap(),
            }
        } else {
            // File has extension but not gz
            OutputFile::Fastq {
                read: std::fs::File::create(name.as_path())
                    .map(bio::io::fastq::Writer::new)
                    .unwrap(),
            }
        }
    } else {
        //file has no extension. Assume plain-text.
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
    umi_sep: Option<&String>,
    edit_nr: Option<u8>,
) -> OutputFile {
    let s = input;
    let delim = umi_sep.as_ref().map(|s| s.as_str()).unwrap_or(":"); // the delimiter for the UMI
    if let Some(number) = edit_nr {
        let header = &[s.id(), delim, std::str::from_utf8(&umi).unwrap()].concat();
        let mut string = String::from(s.desc().unwrap());
        string.replace_range(0..1, &number.to_string());
        let desc: Option<&str> = Some(&string);
        output.write(header, desc, s)
    } else {
        let header = &[s.id(), delim, std::str::from_utf8(&umi).unwrap()].concat();
        output.write(header, s.desc(), s.clone())
    }
}

// Checks whether an output path exists.
pub fn check_outputpath(mut path: PathBuf, compress: &bool, force: &bool) -> Result<PathBuf> {
    // handle the compression and adapt file extension if necessary.
    if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
        if !extension.ends_with("gz") & compress {
            let mut new_extension = extension.to_owned();
            new_extension.push_str(".gz");
            path.set_extension(new_extension);
        }
    }

    // check if the path already exists
    let exists = fs::metadata(&path).is_ok();

    // return the path of it is ok to write, otherwise an error.
    if exists & !force {
        // force will disable prompt, but not the check.
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

pub fn append_umi_to_path(path: &PathBuf) -> PathBuf {
    let path_str = path.as_os_str().clone().to_string_lossy();
    let re = Regex::new(r"^(?P<stem>\.*[^\.]+)\.(?P<extension>.*)$").unwrap();
    let new_path_str = re.replace(&path_str, "${stem}_with_UMIs.${extension}");
    PathBuf::from(new_path_str.to_string())
}
