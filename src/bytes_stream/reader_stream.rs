
use super::BytesStream;
use super::size_limit::SizeLimit;

use std::pin::Pin;
use std::task::{ Context, Poll };

use bytes::{ Bytes, BytesMut };
use tokio::io::{ self, AsyncRead };
use tokio_util::io::poll_read_buf;

use pin_project_lite::pin_project;

// mostly the same as https://docs.rs/tokio-util/0.6.0/src/tokio_util/io/reader_stream.rs.html
pin_project!{
	/// A type that holds an `AsyncRead` and implements `BytesStream`.
	/// 
	/// Can limit the size that is read. If the limit is reached
	/// `SizeLimitReached` get's returned.
	#[derive(Debug)]
	#[allow(dead_code)]
	pub(crate) struct ReaderStream<R> {
		// reader becomes none if size limit surpassed
		// an error ocurred, or all bytes were read
		#[pin]
		reader: Option<R>,// is none if reader ended
		buf: BytesMut,
		size_limit: SizeLimit
	}
}

impl<R: AsyncRead> ReaderStream<R> {

	#[allow(dead_code)]
	pub(crate) fn new(reader: R) -> Self {
		Self {
			reader: Some(reader),
			buf: BytesMut::new(),
			size_limit: SizeLimit::empty()
		}
	}

	// panics if max_buf_size is zero
	#[allow(dead_code)]
	pub(crate) fn limit(reader: R, max_size: usize) -> Self {
		Self {
			reader: Some(reader),
			buf: BytesMut::new(),
			size_limit: SizeLimit::new(max_size)
		}
	}

}

impl<R: AsyncRead> BytesStream for ReaderStream<R> {
	fn poll_bytes(
		mut self: Pin<&mut Self>,
		cx: &mut Context
	) -> Poll<io::Result<Option<Bytes>>> {

		let mut this = self.as_mut().project();

		let reader = match this.reader.as_pin_mut() {
			Some(r) => r,
			None => return Poll::Ready(Ok(None))
		};

		if this.buf.capacity() == 0 {
			let mut new_cap = this.size_limit.new_capacity();

			if new_cap == 0 {
				// the size limit was reached
				// we wan't to know if we read everything
				// or if the size limit was surpassed
				new_cap = 1;
			}

			this.buf.reserve(new_cap);
		}

		// we expect that poll_read_buf does not allocate
		// and it doens't
		match poll_read_buf(reader, cx, &mut this.buf) {
			Poll::Pending => Poll::Pending,
			Poll::Ready(Err(e)) => {
				self.project().reader.set(None);
				Poll::Ready(Err(e))
			},
			Poll::Ready(Ok(0)) => {
				self.project().reader.set(None);
				Poll::Ready(Ok(None))
			},
			Poll::Ready(Ok(s)) => Poll::Ready(
				match this.size_limit.add_read_res(s) {
					Ok(_) => {
						let chunk = this.buf.split().freeze();
						Ok(Some(chunk))
					},
					Err(e) => {
						// sizelimit was surpassed
						self.project().reader.set(None);
						Err(e)
					}
				}
			)
		}
	}
}


#[cfg(test)]
mod tests {

	use super::*;
	use crate::bytes_stream::{BytesStreamExt, StreamReader, SizeLimitReached};

	#[tokio::test]
	async fn test_reader_stream() {

		let read = StreamReader::new(Bytes::from("my body"));
		let mut stream = ReaderStream::new(read);

		let bytes = stream.next_bytes().await.unwrap().unwrap();
		assert_eq!(bytes, &b"my body"[..]);

		assert!(stream.next_bytes().await.unwrap().is_none());
		// check that always none is returned after the stream was read.
		assert!(stream.next_bytes().await.unwrap().is_none());

	}

	#[tokio::test]
	async fn test_size_limit() {

		let read = StreamReader::new(Bytes::from("my body"));
		let mut stream = ReaderStream::limit(read, 2);

		let next_bytes = stream.next_bytes().await;
		let err = next_bytes.unwrap_err();
		let _ = SizeLimitReached::downcast(&err).unwrap();

		assert!(stream.next_bytes().await.unwrap().is_none());

	}

	#[tokio::test]
	async fn test_size_limit_exact() {

		let read = StreamReader::new(Bytes::from("my body"));
		let mut stream = ReaderStream::limit(read, 7);

		let bytes = stream.next_bytes().await.unwrap().unwrap();
		assert_eq!(bytes, &b"my body"[..]);

		assert!(stream.next_bytes().await.unwrap().is_none());
		// check that always none is returned after the stream was read.
		assert!(stream.next_bytes().await.unwrap().is_none());

	}

}