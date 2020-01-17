use core::convert::TryFrom;
use super::alloc::vec::Vec;
use super::error::ParseError;

pub const ELECTRODE_ENABLE_ID: u8 = 0;
pub const DRIVE_ENABLE_ID: u8 = 1;
pub const BULK_CAPACITANCE_ID: u8 = 2;
pub const ACTIVE_CAPACITANCE_ID: u8 = 3;

#[derive(Debug, Clone)]
pub enum Message {
    BulkCapacitanceMsg(BulkCapacitanceStruct),
    ActiveCapacitanceMsg(ActiveCapacitanceStruct),
}

impl Message {
    /// Return the expected payload size for the message, if it can be determined
    /// The size can depend on the data, and so it may not be known until sufficient
    /// bytes are received. 
    pub fn message_size(id: u8, data: &[u8]) -> Option<usize> {
        match id {
            BULK_CAPACITANCE_ID => BulkCapacitanceStruct::message_size(data),
            ACTIVE_CAPACITANCE_ID => ActiveCapacitanceStruct::message_size(data),
            _ => panic!("Invalid message ID {}", id),
        }
    }

    pub fn from_payload(id: u8, data: &[u8]) -> Result<Message, ParseError> {
        use Message::*;
        match id {
            BULK_CAPACITANCE_ID => Ok(BulkCapacitanceMsg(BulkCapacitanceStruct::try_from(data)?)),
            ACTIVE_CAPACITANCE_ID => Ok(ActiveCapacitanceMsg(ActiveCapacitanceStruct::try_from(data)?)),
            _ => panic!("Invalid message ID {}", id),
        }
    }

}

#[derive(Debug, Clone)]
pub struct BulkCapacitanceStruct {
    pub start_index: u8,
    pub values: Vec<u16>,
}

impl BulkCapacitanceStruct {
    pub fn message_size(data: &[u8]) -> Option<usize> {
        // We don't know how long the message will be until we get the first byte
        if data.len() < 2 {
            None
        } else {
            Some((data[1] * 2 + 2) as usize)
        }
    }
}

impl TryFrom<&[u8]> for BulkCapacitanceStruct {
    type Error = ParseError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ParseError);
        }
        let start_index = data[0];
        let count = data[1];
        if data.len() < (2 + count * 2) as usize {
            return Err(ParseError);
        }
        let mut values: Vec<u16> = Vec::with_capacity(count as usize);
        for i in 0..count {
            let x: u16 = data[(i*2+2) as usize] as u16 + ((data[(i*2+3) as usize] as u16) << 8);
            values.push(x);
        }
        Ok(Self{start_index, values})
    }
}

#[derive(Debug, Clone)]
pub struct ActiveCapacitanceStruct {
    pub value: u16
}

impl ActiveCapacitanceStruct {
    pub fn message_size(_data: &[u8]) -> Option<usize> {
        Some(2)
    }
}

impl TryFrom<&[u8]> for ActiveCapacitanceStruct {
    type Error = ParseError;
    
    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ParseError);
        }
        let value = data[0] as u16 + ((data[1] as u16) << 8);
        Ok(Self{value})
    }
}

