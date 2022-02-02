
use crate::bytes_stream::{
	BytesStream, ReaderStream, MoreBytes, StreamHttpBody, StreamReader,
	copy_stream_to_async_write
};
#[cfg(feature = "hyper_body")]
use crate::bytes_stream::HyperBodyStream;

use std::{ fmt, default, mem };
use std::pin::Pin;
#[cfg(feature = "timeout")]
use std::time::Duration;

use tokio::io::{ self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt };
#[cfg(feature = "timeout")]
use tokio::time::timeout;

use bytes::Bytes;

pub type PinnedAsyncRead = Pin<Box<dyn AsyncRead + Send + Sync>>;
pub type PinnedBytesStream = Pin<Box<dyn BytesStream + Send + Sync>>;

pub type FireHttpBody = StreamHttpBody<PinnedBytesStream>;

/// The body for any request or response.
pub enum Body {
	Bytes(Bytes),
	MoreBytes(MoreBytes),
	#[cfg(feature = "hyper_body")]
	/// only available if the feature `hyper_body` is enabled.
	HyperBody(HyperBodyStream),
	AsyncRead(PinnedAsyncRead),
	BytesStream(PinnedBytesStream),
	Empty
}

impl Body {

	/// Creates a new empty body.
	pub fn new() -> Self {
		Self::Empty
	}

	/// Creates a new body with the give bytes, an empty body is returned
	/// if bytes is empty.
	pub fn from_bytes<B>(bytes: B) -> Self
	where B: Into<Bytes> {
		let bytes = bytes.into();
		if bytes.is_empty() {
			Self::Empty
		} else {
			Self::Bytes(bytes)
		}
	}

	/// Creates a new `Body::Bytes` from the given bytes if the slice
	/// is not empty.
	pub fn copy_from_slice(slice: &[u8]) -> Self {
		if slice.is_empty() {
			Self::Empty
		} else {
			Self::Bytes(Bytes::copy_from_slice(slice))
		}
	}

	/// Returns if self is `Body::Empty`.
	pub fn is_empty(&self) -> bool {
		matches!(self, Body::Empty)
	}

	/// Returns this instance and replaces self with `Body::empty`.
	pub fn take(&mut self) -> Self {
		mem::take(self)
	}

	/// Replaces the body with a new one.
	pub fn replace(&mut self, other: Self) -> Self {
		mem::replace(self, other)
	}

	/// Returns a length if it is already known.
	/// 
	/// ## Note
	/// `Body::Empty` is returned as `Some(0)`.
	pub fn len(&self) -> Option<usize> {
		match self {
			Self::Bytes(b) => Some(b.len()),
			Self::MoreBytes(b) => Some(b.len()),
			Self::Empty => Some(0),
			_ => None
		}
	}

	/// Creates a new Body from an `AsyncRead` implementation.
	/// This puts the AsyncRead in a box.
	pub fn from_async_read<R>(reader: R) -> Self
	where R: AsyncRead + Send + Sync + 'static {
		Self::AsyncRead(Box::pin(reader))
	}

	/// Creates a new Body from an `BytesStream` implementation.
	/// This puts the BytesStream in a box.
	pub fn from_bytes_stream<S>(stream: S) -> Self
	where S: BytesStream + Send + Sync + 'static {
		Self::BytesStream(Box::pin(stream))
	}

	/// Creates a new Body from a hyper Body. Aftwards you can set
	/// a size limit with `set_size_limit`.
	/// 
	/// ## Note
	/// Works only with the `hyper_body` feature.
	#[cfg(feature = "hyper_body")]
	pub fn from_hyper_body(body: hyper::Body) -> Self {
		HyperBodyStream::new(body).into()
	}

	/// Sets a read size limit to the HyperBody. Returns true if the size limit
	/// was set.
	/// 
	/// ## Note
	/// Works only with the `hyper_body` feature.  
	/// When the size limit is reached an io::Error::Other with SizeLimitReached
	/// is returned.
	/// 
	/// ## Panics while reading
	/// If the body was already read more than the max_size or the max_size is 0.
	#[cfg(feature = "hyper_body")]
	pub fn set_size_limit(&mut self, max_size: usize) -> bool {
		match self {
			Self::HyperBody(body) => {
				body.set_size_limit(max_size);
				true
			},
			_ => false
		}
	}

	/// Converts the Body to a StreamHttpBody which implements `http_body::Body`,
	/// useful when using hyper.
	pub fn into_http_body(self) -> StreamHttpBody<PinnedBytesStream> {
		let stream: PinnedBytesStream = match self {
			Self::Empty => return StreamHttpBody::new(None),
			a => a.into_bytes_stream()
		};
		StreamHttpBody::new(Some(stream))
	}

	/// Converts the Body to a boxed BytesStream.
	pub fn into_bytes_stream(self) -> PinnedBytesStream {
		match self {
			Self::Bytes(bytes) => Box::pin(bytes),
			Self::MoreBytes(more_bytes) => Box::pin(more_bytes),
			#[cfg(feature = "hyper_body")]
			Self::HyperBody(body) => Box::pin(body),
			Self::AsyncRead(pinned) => Box::pin(ReaderStream::new(pinned)),
			Self::BytesStream(stream) => stream,
			Self::Empty => Box::pin(())
		}
	}

	/// Converts the Body to a boxed AsyncRead.
	pub fn into_async_read(self) -> PinnedAsyncRead {
		match self {
			Self::AsyncRead(read) => read,
			Self::Empty => Box::pin(&[] as &[u8]),
			_ => Box::pin(StreamReader::new(self.into_bytes_stream()))
		}
	}

	/// Converts the body into MoreBytes returning an error if reading
	/// failed or the size limit was reached.
	pub async fn into_more_bytes(self) -> io::Result<MoreBytes> {
		let mut more_bytes = MoreBytes::empty();
		match self {
			Self::Bytes(bytes) => more_bytes.push(bytes),
			Self::MoreBytes(more_bytes) => return Ok(more_bytes),
			#[cfg(feature = "hyper_body")]
			Self::HyperBody(body) =>
				more_bytes.fill_from_stream(body).await?,
			Self::AsyncRead(read) =>
				more_bytes.fill_from_stream(ReaderStream::new(read)).await?,
			Self::BytesStream(stream) =>
				more_bytes.fill_from_stream(stream).await?,
			Self::Empty => {}
		}
		Ok(more_bytes)
	}

	/// Converts the body into a Vector.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.
	pub async fn into_vec(self) -> io::Result<Vec<u8>> {
		match self {
			Self::Bytes(bytes) => Ok(bytes.to_vec()),
			Self::MoreBytes(bytes) => Ok(bytes.to_vec()),
			Self::Empty => Ok(vec![]),
			_ => {
				// TODO check maybe here MoreBytes should be used

				let mut async_read = self.into_async_read();
				let mut vec = Vec::with_capacity(4096);// maybe to big??
				async_read.read_to_end(&mut vec).await?;
				Ok(vec)
			}
		}
	}

	/// Converts the body into a String.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.  
	/// For tests or quick debugging however it is quite suitable.
	pub async fn into_string(self) -> io::Result<String> {
		let v = self.into_vec().await?;
		String::from_utf8(v)
			.map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))
	}

	/// Converts the body into to Body::Bytes, returning the slice.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.
	pub async fn as_slice(&mut self) -> io::Result<&[u8]> {
		match self {
			Self::Bytes(bytes) => return Ok(&*bytes),
			Self::Empty => return Ok(&[]),
			_ => {}
		}
		let v = self.take().into_vec().await?;
		self.replace(Self::Bytes(v.into()));
		match self {
			Self::Bytes(b) => Ok(&*b),
			_ => unreachable!()
		}
	}

	/// Serializes a given type with json creating a Body.
	#[cfg(feature = "json")]
	pub fn serialize<S: ?Sized>(value: &S) -> Result<Self, serde_json::Error>
	where S: serde::Serialize {
		serde_json::to_vec(value).map(|v| v.into())
	}

	/// Tries to deserialize a given Body.
	#[cfg(feature = "json")]
	pub async fn deserialize<D>(self) -> Result<D, JsonError>
	where D: serde::de::DeserializeOwned {
		let more_bytes = self.into_more_bytes().await?;
		// should we add blocking here??
		serde_json::from_reader(more_bytes)
			.map_err(|e| e.into())
		/*from_slice(self.bytes.body())
			.map_err(|e| err!("could not deserialize body {:?}", e))*/
	}

	/// Writes the entire body to an AsyncWrite implementer.
	pub async fn copy_to_async_write<W>(self, writer: &mut W) -> io::Result<()>
	where W: AsyncWrite + Unpin {
		match self {
			Self::Bytes(bytes) => writer.write_all(&*bytes).await?,
			Self::MoreBytes(more_bytes) => more_bytes.copy_to_async_write(writer).await?,
			#[cfg(feature = "hyper_body")]
			Self::HyperBody(body) => copy_stream_to_async_write(body, writer).await?,
			Self::AsyncRead(mut r) => tokio::io::copy(&mut r, writer).await.map(|_| ())?,
			Self::BytesStream(stream) => copy_stream_to_async_write(stream, writer).await?,
			Self::Empty => return Ok(())
		}
		writer.flush().await
	}

	/// Creates a Body with a timeout.
	/// 
	/// ## Note
	/// The timer starts when the first read is performed.
	#[cfg(feature = "timeout")]
	pub fn add_timeout(self, timeout: Duration) -> BodyWithTimeout {
		BodyWithTimeout::new(self, timeout)
	}

}

impl fmt::Debug for Body {
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "{}", match self {
			Self::Bytes(_) => "Bytes()",
			Self::MoreBytes(_) => "MoreBytes()",
			#[cfg(feature = "hyper_body")]
			Self::HyperBody(_) => "HyperBody()",
			Self::AsyncRead(_) => "AsyncRead()",
			Self::BytesStream(_) => "BytesStream()",
			Self::Empty => "Empty"
		})
	}
}

impl default::Default for Body {
	fn default() -> Self {
		Self::Empty
	}
}

impl From<&'static str> for Body {
	fn from(s: &'static str) -> Self {
		Self::from_bytes(s)
	}
}

impl From<String> for Body {
	fn from(s: String) -> Self {
		Self::from_bytes(s)
	}
}

impl From<&'static [u8]> for Body {
	fn from(b: &'static [u8]) -> Self {
		Self::from_bytes(b)
	}
}

impl From<Vec<u8>> for Body {
	fn from(s: Vec<u8>) -> Self {
		Self::from_bytes(s)
	}
}

impl From<()> for Body {
	fn from(_: ()) -> Self {
		Self::Empty
	}
}

#[cfg(feature = "hyper_body")]
impl From<HyperBodyStream> for Body {
	fn from(b: HyperBodyStream) -> Self {
		Self::HyperBody(b)
	}
}

impl From<PinnedAsyncRead> for Body {
	fn from(r: PinnedAsyncRead) -> Self {
		Self::AsyncRead(r)
	}
}

impl From<PinnedBytesStream> for Body {
	fn from(s: PinnedBytesStream) -> Self {
		Self::BytesStream(s)
	}
}

#[cfg(feature = "json")]
mod json_error {

	use super::*;
	use std::error::Error;

	/// The Error returned when deserializing.
	#[derive(Debug)]
	pub enum JsonError {
		IoError(io::Error),
		SerdeJson(serde_json::Error)
	}

	impl From<serde_json::Error> for JsonError {
		fn from(e: serde_json::Error) -> Self {
			Self::SerdeJson(e)
		}
	}

	impl From<io::Error> for JsonError {
		fn from(e: io::Error) -> Self {
			Self::IoError(e)
		}
	}

	impl fmt::Display for JsonError {
		fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
			match self {
				Self::IoError(e) => write!(f, "JsonError::IO({})", e),
				Self::SerdeJson(e) => write!(f, "JsonError::Json({})", e)
			}
		}
	}

	impl Error for JsonError {
		fn source(&self) -> Option<&(dyn Error + 'static)> {
			match self {
				Self::IoError(e) => Some(e),
				Self::SerdeJson(e) => Some(e)
			}
		}
	}
}
#[cfg(feature = "json")]
pub use json_error::*;


/// Adds a timeout to any Body.
#[cfg(feature = "timeout")]
#[derive(Debug)]
pub struct BodyWithTimeout {
	body: Body,
	timeout: Duration
}


#[cfg(feature = "timeout")]
impl BodyWithTimeout {

	/// Creates a new BodyWithTimeout.
	pub(crate) fn new(body: Body, timeout: Duration) -> Self {
		Self {body, timeout}
	}

	/// Creates a BodyWithTimeout from a `hyper::Body`.
	/// 
	/// ## Panics while reading
	/// If the body was already read more than the max_size or the max_size is 0.
	#[cfg(feature = "hyper_body")]
	pub fn from_hyper_body(
		body: hyper::Body,
		max_size: usize,
		timeout: Duration
	) -> Self {
		Self {
			body: HyperBodyStream::limit(body, max_size).into(),
			timeout
		}
	}

	/// Creates a new BodyWithTimeout leaving the old one, with an empty Body.
	pub fn take(&mut self) -> Self {
		Self {
			body: self.body.take(),
			timeout: self.timeout
		}
	}

	/// Returns the underlying Body.
	/// 
	/// ## Note
	/// This removes the timeout.
	pub fn into_body(self) -> Body {
		self.body
	}

	/// Returns a reference to the underlying Body.
	pub fn body(&self) -> &Body {
		&self.body
	}

	/// Returns a mutable reference to the underlying Body.
	/// 
	/// ## Note
	/// Calls to the body won't have a timeout.
	pub fn body_mut(&mut self) -> &mut Body {
		&mut self.body
	}

	/// Checks if the body is empty.
	/// 
	/// ## Note
	/// This only checks if Body is `Body::Empty`.
	pub fn is_empty(&self) -> bool {
		self.body.is_empty()
	}

	/// Sets a read size limit to the HyperBody. Returns true if the size limit
	/// was set.
	/// 
	/// ## Note
	/// Works only with the `hyper_body` feature.  
	/// When the size limit is reached an io::Error::Other with SizeLimitReached
	/// is returned.
	/// 
	/// ## Panics while reading
	/// If the body was already read more than the max_size or the max_size is 0.
	#[cfg(feature = "hyper_body")]
	pub fn set_size_limit(&mut self, max_size: usize) -> bool {
		// Todo add a size limit to ReaderStream
		self.body.set_size_limit(max_size)
	}

	/// Sets the timeout.
	pub fn set_timeout(&mut self, timeout: Duration) {
		self.timeout = timeout;
	}

	/// Converts the body into MoreBytes returning an error if reading
	/// failed or the size limit was reached.
	pub async fn into_more_bytes(self) -> io::Result<MoreBytes> {
		timeout(self.timeout, self.body.into_more_bytes()).await?
	}

	/// Converts the body into a Vector.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.
	pub async fn into_vec(self) -> io::Result<Vec<u8>> {
		timeout(self.timeout, self.body.into_vec()).await?
	}

	/// Converts the body into a String.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.  
	/// For tests or quick debugging however it is quite suitable.
	pub async fn into_string(self) -> io::Result<String> {
		timeout(self.timeout, self.body.into_string()).await
			.map_err(io::Error::from)?
	}

	/// Converts the body into to Body::Bytes, returning the slice.
	/// 
	/// ## Note
	/// If possible, avoid this function as it is really inefficient.
	pub async fn as_slice(&mut self) -> io::Result<&[u8]> {
		timeout(self.timeout, self.body.as_slice()).await?
	}

	/// Tries to deserialize a given Body.
	#[cfg(feature = "json")]
	pub async fn deserialize<D>(self) -> Result<D, JsonError>
	where D: serde::de::DeserializeOwned {
		timeout(self.timeout, self.body.deserialize()).await
			.map_err(io::Error::from)?
	}

	/// Writes the entire body to an AsyncWrite implementer.
	pub async fn copy_to_async_write<W>(self, writer: &mut W) -> io::Result<()>
	where W: AsyncWrite + Unpin {
		timeout(self.timeout, self.body.copy_to_async_write(writer)).await?
	}

}