#![allow(dead_code)]
#![allow(unused_variables)]

extern crate tokio_proto;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
extern crate bytes;
extern crate byteorder;

mod frames;
mod service;
mod protocol;

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

#[cfg(test)]
mod tests {
    #[test]
    fn lib_good() {
    	assert!(true);
    }
}
