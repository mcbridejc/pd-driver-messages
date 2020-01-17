#![no_std]

extern crate alloc;

mod messages;
mod error;

use messages::*;
use error::ParseError;

const MAX_MESSAGE_SIZE: usize = 128;

pub struct WorkingBuffer {
    count: usize,
    buffer: [u8; MAX_MESSAGE_SIZE],
}

pub fn checksum(data: &[u8]) -> (u8, u8) {
    let mut a: u8 = 0;
    let mut b: u8 = 0;
    for x in data {
        a = a.wrapping_add(*x);
        b = b.wrapping_add(b);
    }
    return (a, b);
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
        if self.count >= 1 {
            &self.buffer[1..self.count]
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
            checksum(&self.buffer[0..self.count])
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
            Err(ParseError{})
        }
    }

    pub fn reset(&mut self) {
        self.count = 0;
    }

    pub fn new() -> WorkingBuffer {
        WorkingBuffer{count: 0, buffer: [0; MAX_MESSAGE_SIZE]}
    }
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

    pub fn parse(&mut self, byte: u8) -> Option<Message> {
        let mut byte = byte;
        if self.escaping {
            byte = byte ^ 0x20;
            self.escaping = false;
        } else if byte == 0x7d {
            self.escaping = true;
            return None;
        } else if byte == 0x7e {
            // start of frame
            self.reset();
            return None;
        }

        if let Err(_e) = self.buffer.push(byte) {
            self.reset();
            return None;
        }

        let msg_id = self.buffer.msg_id().unwrap();
        let payload = self.buffer.payload();
        if self.buffer.is_complete() {
            let result = Message::from_payload(msg_id, payload);
            self.reset();
            if result.is_ok() {
                return Some(result.unwrap());
            }
        }
        None
    }
}

#[cfg(test)]
//#[macro_use]
extern crate std;
mod tests {
    use crate::alloc::vec;
    use crate::alloc::vec::Vec;
    use crate::Parser;
    use crate::Message;
    fn append_checksum(data: &mut Vec<u8>) {
        use crate::checksum;
        let (chk_a, chk_b) = checksum(&data[1..data.len()]);
        data.append(&mut vec![chk_a, chk_b]);
    }

    fn parse_message(parser: &mut Parser, data: &[u8]) -> Option<Message> {
        for b in data {
            match parser.parse(*b) {
                Some(msg) => return Some(msg),
                None => (),
            }
        }
        None
    }

    #[test]
    fn test_bulk_capacitance_struct() {
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
    fn test_bulk_capacitance_parse() {
        use crate::*;
        let mut bytes = vec![0x7e, BULK_CAPACITANCE_ID, 0, 2, 04, 0, 05, 0];
        append_checksum(&mut bytes);
        let mut rxmsg = None;
        let mut parser = Parser::new();
        if let Some(msg) = parse_message(&mut parser, &bytes) {
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
        let mut bytes = vec![0x7e, ACTIVE_CAPACITANCE_ID, 2, 3];
        append_checksum(&mut bytes);
        let mut parser = Parser::new();
        if let Some(msg) = parse_message(&mut parser, &bytes) {
            use Message::*;
            match msg {
                
                ActiveCapacitanceMsg(msg) => {
                    assert_eq!(msg.value, 0x302);
                },
                _ => panic!("Invalid message type found"),
            }
        } else {
            panic!("No message parsed");
        }
    }
}
