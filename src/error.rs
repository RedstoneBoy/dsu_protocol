use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub struct UnsupportedProtocolVersion(pub u16);

impl StdError for UnsupportedProtocolVersion {}

impl fmt::Display for UnsupportedProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unsupported protocol version '{}'", self.0)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BufferTooSmall;

impl StdError for BufferTooSmall {}

impl fmt::Display for BufferTooSmall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "buffer too small")?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum HeaderError {
    InvalidMagic([u8; 4]),
    UnsupportedProtocolVersion(u16),
    InvalidMessageType(u32),
}

impl StdError for HeaderError {}

impl fmt::Display for HeaderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "header parse error: ")?;
        match self {
            HeaderError::InvalidMagic(magic) => {
                write!(f, "invalid magic '[{:#X} {:#X} {:#X} {:#X}]'", magic[0], magic[1], magic[2], magic[3])?;
            }
            HeaderError::UnsupportedProtocolVersion(v) => {
                write!(f, "{}", UnsupportedProtocolVersion(*v))?;
            }
            HeaderError::InvalidMessageType(msg_type) => {
                write!(f, "invalid message type '{}'", msg_type)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ControllerInfoError {
    InvalidSlotState(u8),
    InvalidModel(u8),
    InvalidConnectionType(u8),
    InvalidBatteryStatus(u8),
}

impl StdError for ControllerInfoError {}

impl fmt::Display for ControllerInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "controller info message parse error: ")?;
        match self {
            ControllerInfoError::InvalidSlotState(val) => {
                write!(f, "invalid slot state '{}'", val)?;
            }
            ControllerInfoError::InvalidModel(val) => {
                write!(f, "invalid model '{}'", val)?;
            }
            ControllerInfoError::InvalidConnectionType(val) => {
                write!(f, "invalid connection type '{}'", val)?;
            }
            ControllerInfoError::InvalidBatteryStatus(val) => {
                write!(f, "invalid battery status '{:#X}'", val)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RequestControllerInfoError {
    InvalidPortSize(i32),
    InvalidSlot(u8),
    NotEnoughData,
}

impl StdError for RequestControllerInfoError {}

impl fmt::Display for RequestControllerInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "request controller info message parse error: ")?;
        match self {
            RequestControllerInfoError::InvalidPortSize(val) => {
                write!(f, "invalid port size '{}'", val)?;
            }
            RequestControllerInfoError::InvalidSlot(val) => {
                write!(f, "invalid slot '{}'", val)?;
            }
            RequestControllerInfoError::NotEnoughData => {
                write!(f, "not enough data")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RequestControllerDataError {
    InvalidSlot(u8),
    InvalidBitmask(u8),
}

impl StdError for RequestControllerDataError {}

impl fmt::Display for RequestControllerDataError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "request controller data message parse error: ")?;
        match self {
            RequestControllerDataError::InvalidSlot(val) => {
                write!(f, "invalid slot '{}'", val)?;
            }
            RequestControllerDataError::InvalidBitmask(val) => {
                write!(f, "invalid bitmask '{}'", val)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Error {
    UnsupportedProtocolVersion(UnsupportedProtocolVersion),
    Header(HeaderError),
    ControllerInfo(ControllerInfoError),
    RequestControllerInfo(RequestControllerInfoError),
    ReqestControllerData(RequestControllerDataError),
    NotEnoughData,
}

impl StdError for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UnsupportedProtocolVersion(err) => {
                write!(f, "{}", err)?;
            }
            Error::Header(err) => {
                write!(f, "{}", err)?;
            }
            Error::ControllerInfo(err) => {
                write!(f, "{}", err)?;
            }
            Error::RequestControllerInfo(err) => {
                write!(f, "{}", err)?;
            }
            Error::ReqestControllerData(err) => {
                write!(f, "{}", err)?;
            }
            Error::NotEnoughData => {
                write!(f, "not enough data")?;
            }
        }
        Ok(())
    }
}