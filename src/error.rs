use std::{error::Error as StdError, fmt::{Debug, Display, Formatter, Result}};

#[derive(Clone, Debug)]
pub struct Invalid<T>(pub T, pub &'static str);

impl<T: Debug + Display> StdError for Invalid<T> {}

impl<T: Display> Display for Invalid<T> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "invalid value for field '{}': {}", self.1, self.0)?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum RequestControllerInfoError {
    InvalidSlotsLength(i32),
}

impl StdError for RequestControllerInfoError {}

impl Display for RequestControllerInfoError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "RequestControllerInfo parse error: ")?;
        match self {
            RequestControllerInfoError::InvalidSlotsLength(val) => {
                write!(f, "invalid slot length {}", val)?;
            }
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub enum MessageParseError {
    SliceTooSmall,
    InvalidMagic(u32),
    InvalidMessageId(u32),
    InvalidCrc32 {
        expected: u32,
        calculated: u32,
    },
    RequestControllerInfoError(RequestControllerInfoError),
}

impl StdError for MessageParseError {}

impl Display for MessageParseError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "message parse error: ")?;
        match self {
            MessageParseError::SliceTooSmall => {
                write!(f, "slice is too small")?;
            }
            MessageParseError::InvalidMagic(magic) => {
                write!(f, "invalid magic {:#X}", magic)?;
            }
            MessageParseError::InvalidMessageId(val) => {
                write!(f, "invalid message id {}", val)?;
            }
            MessageParseError::InvalidCrc32 { expected, calculated } => {
                write!(f, "invalid crc32, expected {}, calculated {}", expected, calculated)?;
            }
            MessageParseError::RequestControllerInfoError(err) => {
                write!(f, "{}", err)?;
            }
        }

        Ok(())
    }
}

impl From<RequestControllerInfoError> for MessageParseError {
    fn from(err: RequestControllerInfoError) -> Self {
        MessageParseError::RequestControllerInfoError(err)
    }
}