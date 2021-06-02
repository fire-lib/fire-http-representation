
use super::BytesStream;
use super::size_limit::SizeLimit;

use std::pin::Pin;
use std::task::{ Context, Poll };

use bytes::Bytes;
use tokio::io;

use http_body::Body;

/// A wrapper around an `hyper::Body` implementing `BytesStream`.
/// 
/// Also allowing to limit the size that is read before returning
/// an error.
pub struct HyperBodyStream {// should add size limit
	body: hyper::Body,
	size_limit: SizeLimit
}

impl HyperBodyStream {

	pub(crate) fn new(body: hyper::Body) -> Self {
		Self {
			body, size_limit: SizeLimit::empty()
		}
	}

	#[cfg(any(test, all(feature = "timeout", feature = "hyper_body")))]
	pub(crate) fn limit(body: hyper::Body, max: usize) -> Self {
		Self {
			body, size_limit: SizeLimit::new(max)
		}
	}

	pub(crate) fn set_size_limit(&mut self, max_size: usize) {
		self.size_limit.set(max_size)
	}

}

impl BytesStream for HyperBodyStream {
	/// After returning a size limit error None will always be returned.
	fn poll_bytes(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<io::Result<Option<Bytes>>> {
		if self.size_limit.surpassed() {
			return Poll::Ready(Ok(None))
		}

		match Pin::new(&mut self.body).poll_data(cx) {
			Poll::Ready(None) => Poll::Ready(Ok(None)),
			Poll::Ready(Some(Ok(bytes))) if bytes.len() == 0 => Poll::Ready(Ok(None)),
			Poll::Ready(Some(Ok(bytes))) => {
				Poll::Ready(
					self.size_limit.add_read_res(bytes.len())
						.map(|_| Some(bytes))
				)
			},
			Poll::Ready(Some(Err(e))) => Poll::Ready(Err(
				io::Error::new(io::ErrorKind::Other, e)
			)),
			Poll::Pending => Poll::Pending
		}
	}
}


#[cfg(test)]
mod tests {

	use super::*;
	use crate::bytes_stream::{BytesStreamExt, SizeLimitReached};

	#[tokio::test]
	async fn test_hyper_body() {

		let body: hyper::Body = "my body".into();
		let mut stream = HyperBodyStream::new(body);

		let bytes = stream.next_bytes().await.unwrap().unwrap();
		assert_eq!(bytes, &b"my body"[..]);

		assert!(stream.next_bytes().await.unwrap().is_none());
		// check that always none is returned after the stream was read.
		assert!(stream.next_bytes().await.unwrap().is_none());

	}

	#[tokio::test]
	async fn test_size_limit() {

		let body: hyper::Body = "my body".into();
		let mut stream = HyperBodyStream::limit(body, 2);

		let next_bytes = stream.next_bytes().await;
		let err = next_bytes.unwrap_err();
		let _ = SizeLimitReached::downcast(&err).unwrap();

		assert!(stream.next_bytes().await.unwrap().is_none());

	}

	#[tokio::test]
	async fn test_size_limit_exact() {

		let body: hyper::Body = "my body".into();
		let mut stream = HyperBodyStream::limit(body, 7);

		let bytes = stream.next_bytes().await.unwrap().unwrap();
		assert_eq!(bytes, &b"my body"[..]);

		assert!(stream.next_bytes().await.unwrap().is_none());
		// check that always none is returned after the stream was read.
		assert!(stream.next_bytes().await.unwrap().is_none());

	}

}