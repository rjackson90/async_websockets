use tokio_proto::{pipeline, Parse, Serialize};
use tokio_proto::pipeline::Frame::*;
use bytes::buf::BlockBuf;
use bytes::Buf;
use std::{io, str};
use std::io::{Cursor,Read};
use byteorder::BigEndian;

type Frame = pipeline::Frame<WsFrame, WsFrame, io::Error>;

#[derive(Debug)]
pub enum WsFrame {
	// Data Frames
	Text { payload: String },
	Binary { payload: Vec<u8> },
	// Control Frames
	Close { code: u16, reason: String },
	Ping { payload: Vec<u8> },
	Pong { payload: Vec<u8> }
}

pub struct Parser;

impl Parse for Parser {
	type Out = Frame;

	fn parse(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
		// Collect the received bytes until we have enough for a
		// complete frame. (Note: RFC 6455 says receivers aren't 
		// required to buffer a whole frame before beginning processing)
		let data: Vec<u8> = {
			let cursor = buf.buf();
			cursor.bytes()
				.into_iter()
				.map(|byte| *byte)
				.collect()
		};
		if data.len() < 6 {
			// We are a server, so all incoming data is masked, meaning
			// that the smallest valid frame is 6 bytes
			return None;
		}

		let fin = data[0] & 0x80 == 0x80;	// The 0th bit is the FIN flag
		let opcode = data[0] & 0x0f;	// Bits 4-7 are the opcode
		if data[0] & 0x70 != 0 {
			// Right now this server supports no extensions, so if any
			// are present we need to fail the connection
			return Some(Error(io::Error::new(io::ErrorKind::Other, "Unsupported extension")));
		}

		let mask = data[1] & 0x80 == 0x80; // The 8th bit is the MASK flag
		if !mask {
			// The spec says ALL client frames MUST be masked
			return Some(Error(io::Error::new(io::ErrorKind::Other, "Client input is unmasked")));			
		}

		let mut cursor = Cursor::new(data.clone());
		cursor.set_position(2);
		let len:u64 = match data[1] & 0x7f {
			val @ 0 ... 125 => Some(val as u64),
			126 => Some(cursor.read_u16::<BigEndian>() as u64),
			127 => Some(cursor.read_u64::<BigEndian>()),
			_ => None
		}.unwrap();
		let mut mask_key: [u8; 4] = [0; 4]; 
		cursor.read_exact(&mut mask_key).unwrap();
		let mut payload: Vec<u8> = Vec::with_capacity(len as usize);

		// Unmask data
		for n in 0..len as usize{
			payload.insert(n, cursor.read_u8() ^ mask_key[n % 4]);
		}

		// Drop every byte that has been read
		buf.drop(cursor.position() as usize);

		// Return a different message type depending on the opcode
		match opcode {
			0x0 | 0x2 => Some(Message(WsFrame::Binary { payload: payload.clone() })),
			0x1 => Some(Message(WsFrame::Text { payload: String::from_utf8(payload).unwrap() })),
			0x8 => Some(Message(WsFrame::Close { code: 1002, reason: String::from_utf8(payload).unwrap() })),
			0x9 => Some(Message(WsFrame::Ping { payload: payload.clone() })),
			0xA => Some(Message(WsFrame::Pong { payload: payload.clone() })),
			_ => Some(Error(io::Error::new(io::ErrorKind::Other, "Invalid opcode")))
		}
	}

	fn done(&mut self, buf: &mut BlockBuf) -> Option<Frame> {
		unimplemented!()
	}
}

pub struct Serializer;

impl Serialize for Serializer {
	type In = Frame;

	fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
		unimplemented!()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bytes::buf::BlockBuf;
	use bytes::MutBuf;
	use tokio_proto::Parse;
	use tokio_proto::pipeline::Frame::{Message, Error};

	#[test]
	fn mod_good() {
		assert!(true)
	}

	#[test]
	fn single_unmasked_text() {
		let mut test_buf = BlockBuf::default();
		let mut test_parser = Parser {};
		// Taken from RFC 6455 as an example of a text message containing 'Hello'
		let message = vec![	0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f];

		test_buf.write_slice(&message);
		let result = test_parser.parse(&mut test_buf);
		assert!(result.is_some(), "Failed to parse message");

		match result.unwrap() {
			Error(err) => assert!(true),
			data => assert!(false, "expected an error, got {:?}", data)
		};
		
		let remaining = test_buf.len();
		assert!(remaining == message.len(), "Buffer has {} un-consumed bytes after parse", remaining);
	}

	#[test]
	fn single_masked_text() {
		let mut test_buf = BlockBuf::default();
		let mut test_parser = Parser {};
		// Taken from RFC 6455 as an example of a text message containing 'Hello'
		let message = vec![	0x81, 0x85, 0x37, 0xfa, 0x21, 0x3d, 0x7f, 0x9f, 0x4d, 0x51, 0x58 ];

		test_buf.write_slice(&message);
		let result = test_parser.parse(&mut test_buf);
		assert!(result.is_some(), "Failed to parse message");

		let message = match result.unwrap() {
			Message(msg) => msg,
			err => {
				assert!(false, "Parser returned error {:?}", err);
				WsFrame::Close{code: 666, reason: "Bullshit".to_string()}
			}
		};
		let ws_frame = match message {
			WsFrame::Text{ payload } => payload,
			err => {
				assert!(false, "Incorrect WsFrame variant. Got {:?}", err);
				"Bullshit".to_string()
			} 
		};
		assert!(ws_frame == "Hello");

		let remaining = test_buf.len();
		assert!(remaining == 0, "Buffer has {} un-consumed bytes after parse", remaining);

	}
}
