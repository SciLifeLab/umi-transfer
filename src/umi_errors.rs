use std::path::PathBuf;

#[derive(Debug)]
pub enum RuntimeErrors {
    FileExists(Option<PathBuf>),
    FileNotFound(Option<PathBuf>),
    OutputNotWriteable(Option<PathBuf>),
    ReadIDMismatch,
    ReadWriteError(bio::io::fastq::Record),
}

impl std::fmt::Display for RuntimeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileExists(None) => {
                write!(f, "Output file exists, but must not be overwritten.")
            }
            Self::FileExists(Some(path)) => write!(
                f,
                "Output file {} exists, but must not be overwritten.",
                path.display()
            ),
            Self::FileNotFound(None) => {
                write!(f, "Specified file does not exist or is not readable!")
            }
            Self::FileNotFound(Some(path)) => {
                write!(f, "{} does not exist or is not readable!", path.display())
            }
            Self::OutputNotWriteable(None) => {
                write!(f, "Output file is missing or not writeable.")
            }
            Self::OutputNotWriteable(Some(path)) => write!(
                f,
                "Output file {} is missing or not writeable.",
                path.display()
            ),
            Self::ReadIDMismatch => write!(
                f,
                "IDs of UMI and read records mismatch. Please provide sorted files as input!"
            ),
            Self::ReadWriteError(record) => {
                write!(f, "Failure to write read {} to file.", record.id())
            }
        }
    }
}
