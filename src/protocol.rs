

// WsProtocol is a state machine which tracks the state of WebSocket protocol connections
pub struct WsProtocol<S> {
	state: S
}

pub struct Connecting {}

pub struct Open {}

pub struct Closing {}

pub struct Closed {}

impl WsProtocol<Connecting> {
	pub fn new() -> Self {
		WsProtocol {
			state: Connecting {}
		}
	}
}

impl From<WsProtocol<Connecting>> for WsProtocol<Open> {
	fn from(val: WsProtocol<Connecting>) -> WsProtocol<Open> {
		WsProtocol {
			state: Open {}
		}
	}
}

impl From<WsProtocol<Open>> for WsProtocol<Closing> {
	fn from(val: WsProtocol<Open>) -> WsProtocol<Closing> {
		WsProtocol {
			state: Closing {}
		}
	}
}

impl From<WsProtocol<Closing>> for WsProtocol<Closed> 
{
	fn from(val: WsProtocol<Closing>) -> WsProtocol<Closed> {
		WsProtocol {
			state: Closed {}
		}
	}
}

impl From<WsProtocol<Open>> for WsProtocol<Closed> 
{
	fn from(val: WsProtocol<Open>) -> WsProtocol<Closed> {
		WsProtocol {
			state: Closed {}
		}
	}
}

impl From<WsProtocol<Connecting>> for WsProtocol<Closed> 
{
	fn from(val: WsProtocol<Connecting>) -> WsProtocol<Closed> {
		WsProtocol {
			state: Closed {}
		}
	}
}
