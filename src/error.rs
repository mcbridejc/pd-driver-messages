use core::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    SizeOverrun,
    ChecksumError(u16, u16),
    UnknownPacketId(u8),
    DeserializationError
}

impl fmt::Display for ParseError {
    
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;
        match self {
            SizeOverrun => {
                write!(f, "Tried to parse packet longer than max length")
            },
            ChecksumError(found, exp) => {
                write!(f, "Mismatched checksum. Found {:x}, expected {:x}", found, exp)
            },
            UnknownPacketId(id) => {
                write!(f, "Found unrecognized packet id 0x{:x}", id)
            },
            DeserializationError => {
                write!(f, "Failed parsing payload into packet struct")
            },
        }
    }
}
