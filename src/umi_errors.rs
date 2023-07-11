use std::path::PathBuf;

#[derive(Debug)]
pub enum RuntimeErrors {
    ReadIDMismatchError,
    FileExistsError(Option<PathBuf>),
    FileNotFoundError(Option<PathBuf>),
    OutputNotWriteableError(Option<PathBuf>),
    //GeneralError,
}

impl std::fmt::Display for RuntimeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadIDMismatchError => write!(
                f,
                "IDs of UMI and read records mismatch. Please provide sorted files as input!"
            ),
            Self::FileExistsError(None) => {
                write!(f, "Output file exists, but must not be overwritten.")
            }
            Self::FileExistsError(Some(path)) => write!(
                f,
                "Output file {} exists, but must not be overwritten.",
                path.to_string_lossy()
            ),
            Self::FileNotFoundError(None) => {
                write!(f, "Specified file does not exist or is not readable!")
            }
            Self::FileNotFoundError(Some(path)) => {
                write!(
                    f,
                    "{} does not exist or is not readable!",
                    path.to_string_lossy()
                )
            }
            Self::OutputNotWriteableError(None) => {
                write!(f, "Output file is missing or not writeable.")
            }
            Self::OutputNotWriteableError(Some(path)) => write!(
                f,
                "Output file {} is missing or not writeable.",
                path.to_string_lossy()
            ),
            //Self::GeneralError => write!(f, "Encountered an error."),
        }
    }
}
