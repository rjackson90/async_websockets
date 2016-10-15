use tokio_proto::{pipeline, Parse, Serialize};
use tokio_proto::pipeline::Frame::*;
use bytes::buf::BlockBuf;
use bytes::Buf;
use std::{io, str};
use std::io::{Cursor,Read};
use byteorder::BigEndian;

type Frame = pipeline::Frame<WsFrame, WsFrame, io::Error>;

enum WsFrame {
	// Data Frames
	Text { payload: String },
	Binary { payload: Vec<u8> },
	// Control Frames
	Close { code: u16, reason: String },
	Ping { payload: Vec<u8> },
	Pong { payload: Vec<u8> }
}

struct Parser;

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

		let fin = data[0] & 0x80 == 1;	// The 0th bit is the FIN flag
		let opcode = data[0] & 0x0f;	// Bits 4-7 are the opcode
		if data[0] & 0x70 != 0 {
			// Right now this server supports no extensions, so if any
			// are present we need to fail the connection
			return Some(Error(io::Error::new(io::ErrorKind::Other, "Unsupported extension")));
		}

		let mask = data[1] & 0x80 == 1; // The 8th bit is the MASK flag
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
			payload[n] = cursor.read_u8() ^ mask_key[n % 4];
		}

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

struct Serializer;

impl Serialize for Serializer {
	type In = Frame;

	fn serialize(&mut self, frame: Frame, buf: &mut BlockBuf) {
		unimplemented!()
	}
}

#[cfg(test)]
mod tests {

	#[test]
	fn mod_good() {
		assert!(true)
	}
}
