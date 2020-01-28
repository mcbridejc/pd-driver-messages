#![no_std]

extern crate alloc;

use self::alloc::vec::Vec;
pub mod messages;
mod error;

use messages::*;
use error::ParseError;

const MAX_MESSAGE_SIZE: usize = 128;

pub struct WorkingBuffer {
    count: usize,
    buffer: [u8; MAX_MESSAGE_SIZE],
}

#[derive(Clone, Debug, Default)]
pub struct Checksum {
    pub a: u8,
    pub b: u8,
}

impl Checksum {
    pub fn add_byte(&mut self, x: u8) {
        self.a = self.a.wrapping_add(x);
        self.b = self.b.wrapping_add(self.a);
    }

    pub fn get(&self) -> (u8, u8) {
        (self.a, self.b)
    }
}

pub fn checksum(data: &[u8]) -> (u8, u8) {
    let mut chk = Checksum::default();
    for x in data {
        chk.add_byte(*x);
    }
    chk.get()
}

impl<'a> WorkingBuffer {
    pub fn msg_id(&self) -> Option<u8> {
        if self.count > 0 {
            Some(self.buffer[0])
        } else {
            None
        }
    }

    pub fn payload(&'a self) -> &'a [u8] {
        if self.count >= 3 {
            &self.buffer[1..self.count - 2]
        } else {
            &self.buffer[0..0]
        }
    }

    pub fn checksum(&self) -> (u8, u8) {
        if self.count < 3 {
            (0, 0)
        } else {
            let a = self.buffer[self.count - 2];
            let b = self.buffer[self.count - 1];
            (a, b)
        }
    }

    pub fn calc_checksum(&self) -> (u8, u8) {
        if self.count > 0 {
            checksum(&self.buffer[0..self.count-2])
        } else {
            (0, 0)
        }
    }

    pub fn is_complete(&self) -> bool {
        let msg_id = match self.msg_id() {
            Some(id) => id,
            None => return false,
        };
        let expected_payload_size = Message::message_size(msg_id, self.payload());
        // Expect payload + 1 type + 2 checksum bytes
        if expected_payload_size.is_some() && self.count == expected_payload_size.unwrap() + 3 {
            true
        } else {
            false
        }
    }

    pub fn push(&mut self, byte: u8) -> Result<(), ParseError> {
        if self.count < MAX_MESSAGE_SIZE {
            self.buffer[self.count] = byte;
            self.count += 1;
            Ok(())
        } else {
            Err(ParseError::SizeOverrun)
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn new() -> WorkingBuffer {
        WorkingBuffer{count: 0, buffer: [0; MAX_MESSAGE_SIZE]}
    }
}

/// Get transmittable bytes for msg
pub fn serialize_msg<T>(msg: &T) -> Vec<u8>
where
    T: MessageStruct
{
    let id = msg.id();
    let payload: Vec<u8> = msg.payload();
    serialize_raw(id, &payload)
}

pub fn serialize_raw(id: u8, payload: &[u8]) -> Vec<u8> {
    fn escaped_push(b: u8, buf: &mut Vec<u8>) {
        if b == 0x7d || b == 0x7e {
            buf.push(0x7d);
            buf.push(b ^ 0x20);
        } else {
            buf.push(b);
        }
    }
    // We don't know the size required yet, but we know it will be *at least* this much
    let mut buf = Vec::with_capacity(payload.len() + 4);
    let mut chk = Checksum::default();
    buf.push(0x7e); // Start of frame
    escaped_push(id, &mut buf);
    chk.add_byte(id);
    for b in payload {
        escaped_push(*b, &mut buf);
        chk.add_byte(*b);
    }
    let (chk_a, chk_b) = chk.get();
    escaped_push(chk_a, &mut buf);
    escaped_push(chk_b, &mut buf);
    buf
}

pub struct Parser {
    parsing: bool,
    escaping: bool,
    buffer: WorkingBuffer,
}

impl Parser {
    pub fn new() -> Parser {
        Parser{
            buffer: WorkingBuffer::new(),
            parsing: false,
            escaping: false,
        }
    }

    pub fn reset(&mut self) {
        self.escaping = false;
        self.parsing = false;
        self.buffer.reset();
    }

    pub fn parse(&mut self, byte: u8) -> Result<Option<Message>, ParseError> {
        let mut byte = byte;
        if self.escaping {
            byte = byte ^ 0x20;
            self.escaping = false;
        } else if byte == 0x7d {
            self.escaping = true;
            return Ok(None);
        } else if byte == 0x7e {
            // start of frame
            self.reset();
            return Ok(None);
        }

        if let Err(_e) = self.buffer.push(byte) {
            self.reset();
            return Ok(None);
        }

        if self.buffer.is_complete() {
            if self.buffer.checksum() == self.buffer.calc_checksum() {
                let msg_id = self.buffer.msg_id().unwrap();
                let payload = self.buffer.payload();
                let result = Message::from_payload(msg_id, payload);
                self.reset();
                if result.is_ok() {
                    return Ok(Some(result.unwrap()));
                }
            } else {
                let (found_a, found_b) = self.buffer.checksum();
                let (exp_a, exp_b) = self.buffer.calc_checksum();
                let found = (found_a as u16) + (found_b as u16) * 256;
                let exp = (exp_a as u16) + (exp_b as u16) * 256;
                self.reset();
                return Err(ParseError::ChecksumError(found, exp));
            }
        } 
        Ok(None)
    }
}

#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use crate::alloc::vec;
    use crate::alloc::vec::Vec;
    use crate::Parser;
    use crate::Message;
    use crate::ParseError;
    fn append_checksum(data: &mut Vec<u8>) {
        use crate::checksum;
        let (chk_a, chk_b) = checksum(&data[1..data.len()]);
        std::println!("Appending {:x} {:x} to packet", chk_a, chk_b);
        data.append(&mut vec![chk_a, chk_b]);
    }

    fn parse_message(parser: &mut Parser, data: &[u8]) -> Result<Option<Message>, ParseError> {
        for b in data {
            let result = parser.parse(*b)?;
            match result {
                Some(msg) => return Ok(Some(msg)),
                None => (),
            }
        }
        Ok(None)
    }


    #[test]
    fn test_bulk_capacitance_parse() {
        use crate::*;
        let mut bytes = vec![0x7e, BULK_CAPACITANCE_ID, 0, 2, 04, 0, 05, 0];
        append_checksum(&mut bytes);
        let mut rxmsg = None;
        let mut parser = Parser::new();
        let result = parse_message(&mut parser, &bytes);
        if result.is_err() {
            panic!("Error while parsing: {}", result.err().unwrap());
        }
        let result = result.unwrap();
        if let Some(msg) = result {
            use Message::*;
            match msg {
                BulkCapacitanceMsg(msg) => rxmsg = Some(msg),
                _ => panic!("Got unexpected  messaged: {:?}", msg),
            }
        }
        assert!(rxmsg.is_some());
        let rxmsg = rxmsg.unwrap();
        assert_eq!(rxmsg.values.len(), 2);
        assert_eq!(rxmsg.values[0], 4);
        assert_eq!(rxmsg.values[1], 5);
    }

    #[test]
    fn test_active_capacitance_parse() {
        use crate::*;
        let mut bytes = vec![0x7e, ACTIVE_CAPACITANCE_ID, 2, 3, 4, 5];
        append_checksum(&mut bytes);
        let mut parser = Parser::new();
        let result = parse_message(&mut parser, &bytes);
        if result.is_err() {
            panic!("Error while parsing: {}", result.err().unwrap());
        }
        let result = result.unwrap();
        if let Some(msg) = result {
            use Message::*;
            match msg {
                ActiveCapacitanceMsg(msg) => {
                    assert_eq!(msg.baseline, 0x302);
                    assert_eq!(msg.measurement, 0x504);
                },
                _ => panic!("Invalid message type found"),
            }
        } else {
            panic!("No message parsed");
        }
    }

    #[test]
    fn test_electrode_enable_roundtrip() {
        use crate::*;
        let values: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0x7d, 0x7e];
        let tx_msg = ElectrodeEnableStruct{ values };
        let payload: Vec<u8> = tx_msg.payload();
        let tx_bytes = serialize_raw(ELECTRODE_ENABLE_ID, &payload);
        let mut parser = Parser::new();
        let result = parse_message(&mut parser, &tx_bytes);
        if result.is_err() {
            panic!("Error while parsing: {}", result.err().unwrap());
        }
        let rx_msg = result.unwrap();
        assert!(rx_msg.is_some());
        let rx_msg = rx_msg.unwrap();
        if let Message::ElectrodeEnableMsg(msg) = rx_msg {
            assert_eq!(msg.values, values);
        } else {
            panic!("Did not parse expected message");
        }
    }
}
