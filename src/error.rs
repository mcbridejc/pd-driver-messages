use core::fmt;

#[derive(Debug, Clone)]
pub enum ParseError {
    SizeOverrun,
    ChecksumError(u16, u16),
    UnknownPacketId(u8),
    DeserializationError
}

// #[derive(Debug, Clone)]
// pub struct ParseError;

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

// impl fmt::Display for ParseError::SizeOverrun {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
//     }
// }

// impl fmt::Display for ParseError::ChecksumError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Mismatched checksum. Found {:x}, expected {:x}", self.0, self.1)
//     }
// }


// impl fmt::Display for ParseError::UnknownPacketId {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Found unrecognized packet id 0x{:x}", self.0)
//     }
// }


// impl fmt::Display for ParseError::DeserializationError {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Failed parsing payload into packet struct")
//     }
// }