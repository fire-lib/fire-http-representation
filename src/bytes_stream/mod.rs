
use std::pin::Pin;
use std::future::Future;
use std::task::{ Context, Poll };
use std::mem;

use bytes::Bytes;
use tokio::io::{ self, AsyncWrite, AsyncWriteExt };

mod reader_stream;
pub(crate) use reader_stream::ReaderStream;

mod stream_reader;
pub(crate) use stream_reader::StreamReader;

#[cfg(feature = "hyper_body")]
mod hyper_body_stream;
#[cfg(feature = "hyper_body")]
pub use hyper_body_stream::HyperBodyStream;

mod stream_http_body;
pub use stream_http_body::StreamHttpBody;

mod size_limit;
pub use size_limit::SizeLimitReached;

mod more_bytes;
pub use more_bytes::MoreBytes;

// same as default page size
const DEF_CAPACITY: usize = 4096;

/// A stream that returns Bytes.
pub trait BytesStream {
	/// The returned bytes are never allowed to be empty.
	fn poll_bytes(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<Option<Bytes>>>;
}

impl<S: BytesStream + ?Sized> BytesStream for Pin<Box<S>> {
	fn poll_bytes(self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<Option<Bytes>>> {
		self.get_mut().as_mut().poll_bytes(cx)
	}
}

impl BytesStream for () {
	fn poll_bytes(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<Option<Bytes>>> {
		Poll::Ready(Ok(None))
	}
}

/// An extension trait implemented for all BytesStream types.
pub trait BytesStreamExt: BytesStream {
	/// returns the next bytes.
	/// 
	/// Equivalent to:
	/// ```ignore
	/// async fn next_bytes(&mut self) -> io::Result<Option<Bytes>>;
	/// ```
	fn next_bytes<'a>(&'a mut self) -> NextBytes<'a, Self>
	where Self: Unpin {
		NextBytes { stream: Pin::new(self) }
	}
}

impl<S: BytesStream + ?Sized> BytesStreamExt for S {}

#[doc(hidden)]
#[derive(Debug)]
pub struct NextBytes<'a, S: ?Sized> {
	stream: Pin<&'a mut S>
}

impl<S> Future for NextBytes<'_, S>
where S: BytesStream {

	type Output = io::Result<Option<Bytes>>;

	fn poll(
		self: Pin<&mut Self>,
		cx: &mut Context
	) -> Poll<Self::Output> {
		self.get_mut().stream.as_mut().poll_bytes(cx)
	}

}




impl BytesStream for Bytes {
	fn poll_bytes(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<Option<Bytes>>> {
		let this = self.get_mut();
		Poll::Ready(Ok({
			if this.is_empty() {
				None
			} else {
				Some(mem::take(this))
			}
		}))
	}
}

/// Copies `Bytes` from a `BytesStream` to an `AsyncWrite` implementor.
/// 
/// Consuming the `BytesStream` since it won't have any `Bytes` left.
pub async fn copy_stream_to_async_write<S, W>(mut stream: S, writer: &mut W) -> io::Result<()>
where
	S: BytesStream + Unpin,
	W: AsyncWrite + Unpin {
	while let Some(bytes) = stream.next_bytes().await? {
		writer.write_all(&*bytes).await?
	}
	Ok(())
}





// write tests
#[cfg(test)]
mod tests {

	use super::*;
	use http_body::Body;

	#[tokio::test]
	async fn test_bytes_stream_for_bytes() {

		let mut bytes = Bytes::from_static(b"A little Bytes test");
		let len = bytes.len();
		assert_eq!(bytes.next_bytes().await.unwrap().unwrap().len(), len);
		assert!(bytes.next_bytes().await.unwrap().is_none());

	}

	#[tokio::test]
	async fn test_stream_http_body_with_bytes() {
		// StreamHttpBody

		let bytes = Bytes::from_static(b"A little Bytes test");
		let len = bytes.len();
		let mut body = StreamHttpBody::new(Some(bytes));
		assert_eq!(body.data().await.unwrap().unwrap().len(), len);
		assert!(body.data().await.is_none());

	}

}