use core::convert::TryFrom;
use super::alloc::vec::Vec;
use super::error::ParseError;

pub const ELECTRODE_ENABLE_ID: u8 = 0;
pub const DRIVE_ENABLE_ID: u8 = 1;
pub const BULK_CAPACITANCE_ID: u8 = 2;
pub const ACTIVE_CAPACITANCE_ID: u8 = 3;

#[derive(Debug, Clone)]
pub enum Message {
    ElectrodeEnableMsg(ElectrodeEnableStruct),
    BulkCapacitanceMsg(BulkCapacitanceStruct),
    ActiveCapacitanceMsg(ActiveCapacitanceStruct),
}

impl Message {
    /// Return the expected payload size for the message, if it can be determined
    /// The size can depend on the data, and so it may not be known until sufficient
    /// bytes are received.
    pub fn message_size(id: u8, data: &[u8]) -> Option<usize> {
        match id {
            ELECTRODE_ENABLE_ID => ElectrodeEnableStruct::message_size(data),
            BULK_CAPACITANCE_ID => BulkCapacitanceStruct::message_size(data),
            ACTIVE_CAPACITANCE_ID => ActiveCapacitanceStruct::message_size(data),
            _ => panic!("Invalid message ID {}", id),
        }
    }

    pub fn from_payload(id: u8, data: &[u8]) -> Result<Message, ParseError> {
        use Message::*;
        match id {
            ELECTRODE_ENABLE_ID => Ok(ElectrodeEnableMsg(ElectrodeEnableStruct::try_from(data)?)),
            BULK_CAPACITANCE_ID => Ok(BulkCapacitanceMsg(BulkCapacitanceStruct::try_from(data)?)),
            ACTIVE_CAPACITANCE_ID => Ok(ActiveCapacitanceMsg(ActiveCapacitanceStruct::try_from(data)?)),
            _ => panic!("Invalid message ID {}", id),
        }
    }




}

#[derive(Debug, Clone)]
pub struct ElectrodeEnableStruct {
    pub values: [u8; 16],
}

impl ElectrodeEnableStruct {
    pub fn message_size(_data: &[u8]) -> Option<usize> {
        Some(16)
    }
}

impl Into<Vec<u8>> for ElectrodeEnableStruct {
    fn into(self) -> Vec<u8> {
        self.values[..].into()
    }
}

impl TryFrom<&[u8]> for ElectrodeEnableStruct {
    type Error = ParseError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() != 16 {
            return Err(ParseError);
        }
        let mut values = [0u8; 16];
        for i in 0..16 {
            values[i] = data[i];
        }
        Ok(Self{values})
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

impl Into<Vec<u8>> for BulkCapacitanceStruct {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(self.values.len() * 2 + 2);
        buf.push(self.start_index);
        buf.push(self.values.len() as u8);
        for x in self.values {
            buf.push((x & 0xff) as u8);
            buf.push((x >> 8) as u8);
        }
        buf
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
    pub baseline: u16,
    pub measurement: u16,
}

impl ActiveCapacitanceStruct {
    pub fn message_size(_data: &[u8]) -> Option<usize> {
        Some(4)
    }
}

impl Into<Vec<u8>> for ActiveCapacitanceStruct {
    fn into(self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::with_capacity(4);
        buf.push((self.baseline & 0xff) as u8);
        buf.push((self.baseline >> 8) as u8);
        buf.push((self.measurement & 0xff) as u8);
        buf.push((self.measurement >> 8) as u8);
        buf
    }
}

impl TryFrom<&[u8]> for ActiveCapacitanceStruct {
    type Error = ParseError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 4 {
            return Err(ParseError);
        }
        let baseline = data[0] as u16 + ((data[1] as u16) << 8);
        let measurement = data[2] as u16 + ((data[3] as u16) << 8);
        Ok(Self{baseline, measurement})
    }
}

#[cfg(test)]
mod tests {
    use crate::alloc::vec::Vec;
    #[test]
    fn test_bulk_capacitance_deser() {
        use crate::*;
        let bytes = &[0, 2, 4, 0, 5, 0];
        let message = Message::from_payload(BULK_CAPACITANCE_ID, bytes);
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::BulkCapacitanceMsg(msg) => {
                assert_eq!(msg.start_index, 0);
                assert_eq!(msg.values.len(), 2);
                assert_eq!(msg.values[0], 4);
                assert_eq!(msg.values[1], 5);
            },
            _ => panic!("Wrong kind of message"),
        }
    }
    #[test]
    fn test_bulk_capacitance_ser() {
        use crate::*;
        let expected_bytes = &[8, 2, 4, 0, 5, 0];
        let message = BulkCapacitanceStruct{start_index: 8, values: vec![4, 5]};
        let bytes: Vec<u8> = message.into();
        assert_eq!(bytes, expected_bytes);
    }

    #[test]
    fn test_electrode_enable_deser() {
        use crate::*;
        let bytes = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let message = Message::from_payload(ELECTRODE_ENABLE_ID, bytes);
        assert!(message.is_ok());
        let message = message.unwrap();
        match message {
            Message::ElectrodeEnableMsg(msg) => {
                for i in 0..16 {
                    assert_eq!(msg.values[i], i as u8);
                }
            },
            _ => panic!("Wrong kind of message"),
        }
    }

    #[test]
    fn test_electrode_en_ser() {
        use crate::*;
        let expected_bytes = &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];;
        let message = ElectrodeEnableStruct{values: *expected_bytes};
        let bytes: Vec<u8> = message.into();
        assert_eq!(bytes, expected_bytes);
    }

}