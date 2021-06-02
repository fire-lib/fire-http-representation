
use crate::header::{ ResponseHeaderBuilder, Version, StatusCode, ContentType, HeaderValues, IntoHeaderValue };
use crate::body::Body;
use super::Response;

use http as raw;


// TODO probably remove the http version.

/// A builder to create a `Response`.
#[derive(Debug)]
pub struct ResponseBuilder {
	header: ResponseHeaderBuilder,
	body: Body
}

impl ResponseBuilder {

	/// Creates a new `ResponseBuilder`.
	pub fn new() -> Self {
		Self {
			header: ResponseHeaderBuilder::new(),
			body: Body::new()
		}
	}

	/// Sets the http version.
	pub fn version(mut self, version: Version) -> Self {
		self.header.version(version);
		self
	}

	/// Sets the status code.
	pub fn status_code(mut self, status_code: StatusCode) -> Self {
		self.header.status_code(status_code);
		self
	}

	/// Sets the content type.
	pub fn content_type<T>(mut self, content_type: T) -> Self
	where T: Into<ContentType> {
		self.header.content_type(content_type.into());
		self
	}

	/// Sets a header value.
	/// 
	/// ## Note
	/// Only ASCII characters are allowed, use `self.values_mut().encode()`
	/// to allow any character.
	/// 
	/// ## Panics
	/// If the value is not a valid `HeaderValue`.
	pub fn header<K, V>(mut self, key: K, val: V) -> Self
	where
		K: raw::header::IntoHeaderName,
		V: IntoHeaderValue {
		self.values_mut().insert(key, val);
		self
	}

	/// Returns `HeaderValues` mutably.
	pub fn values_mut(&mut self) -> &mut HeaderValues {
		self.header.values_mut()
	}

	/// Sets the body dropping the previous one.
	pub fn body<B>(mut self, body: B) -> Self
	where B: Into<Body> {
		self.body.replace(body.into());
		self
	}

	/*pub fn body_reader<R>(mut self, reader: R) -> Self
	where R: AsyncRead + Send + Sync + 'static {
		self.body = Some(Body::from_reader(reader));
		self
	}*/

	/// Builds a `Response`. Adding the `content-length` header
	/// if the len of the body is known.
	pub fn build(mut self) -> Response {
		// lets calculate content-length
		// if the body size is already known
		if let Some(len) = self.body.len() {
			self.values_mut().insert("content-length", len);
		}
		Response::new(self.header.build(), self.body)
	}

}