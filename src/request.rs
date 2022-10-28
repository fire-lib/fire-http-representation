use crate::header::RequestHeader;
use crate::body::Body;

/// The request that is received from a client.
#[derive(Debug)]
pub struct Request {
	pub header: RequestHeader,
	pub body: Body
}

impl Request {
	/// Creates a new `Request`.
	pub fn new(header: RequestHeader, body: Body) -> Self {
		Self { header, body }
	}

	/// Takes the body replacing it with an empty one.
	pub fn take_body(&mut self) -> Body {
		self.body.take()
	}

	/// Get the request header by reference.
	pub fn header(&self) -> &RequestHeader {
		&self.header
	}

	/// Get a mutable request header.
	pub fn header_mut(&mut self) -> &mut RequestHeader {
		&mut self.header
	}
}