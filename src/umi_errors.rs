#[derive(Debug)]
pub enum RuntimeErrors {
    ReadIDMismatchError,
    FileNotFoundError,
    FileExistsError,
    //GeneralError,
}

impl std::fmt::Display for RuntimeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadIDMismatchError => write!(
                f,
                "IDs of UMI and read records mismatch. Please provide sorted files!"
            ),
            Self::FileNotFoundError => {
                write!(f, "Specified file does not exist or is not readable!")
            }
            Self::FileExistsError => write!(f, "Output file exists, but must not be overwritten."),
            //Self::GeneralError => write!(f, "Encountered an error."),
        }
    }
}
