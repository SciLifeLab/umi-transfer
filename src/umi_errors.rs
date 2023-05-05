#[derive(Debug)]
pub enum RuntimeErrors {
    ReadIDMismatchError,
    FileNotFoundError,
    GeneralError,
}

impl std::fmt::Display for RuntimeErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ReadIDMismatchError => write!(
                f,
                "IDs of UMI and read records mismatch. Please provide sorted files!"
            ),
            Self::FileNotFoundError => write!(f, "Cannot read from specified path."),
            Self::GeneralError => write!(f, "Encountered an error."),
        }
    }
}
