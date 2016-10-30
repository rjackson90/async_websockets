use tokio_proto::{pipeline, server, Message};
use tokio_service::{Service, NewService};
use tokio_core::reactor::Handle;
use futures::{Async, BoxFuture};
use futures::stream::Empty;
use std::{io};
use std::net::SocketAddr;

use super::WsFrame;
use super::frames::new_ws_transport;


struct WebSocketService<T> {
	inner: T
}

impl<T> Service for WebSocketService<T>
	where T: Service<Request = WsFrame, Response = WsFrame, Error = io::Error>,
		T::Future: 'static
{
	type Request = WsFrame;
	type Response = Message<WsFrame, Empty<(), io::Error>>;
	type Error = io::Error;
	type Future = BoxFuture<Self::Response, Self::Error>;

	fn call(&self, req: Self::Request) -> Self::Future {
		unimplemented!()
	}

	fn poll_ready(&self) -> Async<()> {
		Async::Ready(())
	}
}


pub fn serve<T>(handle: &Handle, addr: SocketAddr, client_service: T)
	-> io::Result<()>
	where T: NewService<
		  Request = WsFrame
		, Response = WsFrame
		, Error = io::Error>
	+ Send
	+ 'static
{
	try!(server::listen(handle, addr, move |stream| {
		let service = WebSocketService{ inner: try!(client_service.new_service()) };
		pipeline::Server::new(service, new_ws_transport(stream))
	}));
	Ok(())
}
