
use super::BytesStream;

use std::pin::Pin;
use std::task::{ Context, Poll };

use bytes::{ Bytes, Buf };
use tokio::io::{ self, AsyncRead, ReadBuf };

use pin_project_lite::pin_project;

// mostly the same as https://docs.rs/tokio-util/0.6.0/src/tokio_util/io/stream_reader.rs.html
pin_project!{
	#[derive(Debug)]
	pub struct StreamReader<S> {
		#[pin]
		stream: Option<S>,// is none if stream ended
		active: Bytes
	}
}

impl<S: BytesStream> StreamReader<S> {

	pub(crate) fn new(stream: S) -> Self {
		Self {
			stream: Some(stream),
			active: Bytes::new()
		}
	}

	fn fill_buf_from_active(buf: &mut ReadBuf<'_>, active: &mut Bytes) {
		debug_assert!(buf.remaining() != 0);
		debug_assert!(active.len() != 0);

		let len = buf.remaining().min(active.len());
		buf.put_slice(&active[..len]);
		active.advance(len);
	}

}

impl<S: BytesStream> AsyncRead for StreamReader<S> {
	fn poll_read(
		mut self: Pin<&mut Self>,
		cx: &mut Context,
		buf: &mut ReadBuf<'_>
	) -> Poll<io::Result<()>> {
		if buf.remaining() == 0 {
			return Poll::Ready(Ok(()));
		}

		let this = self.as_mut().project();

		if this.active.len() != 0 {
			Self::fill_buf_from_active(buf, this.active);
			return Poll::Ready(Ok(()));
		}


		let stream = match this.stream.as_pin_mut() {
			Some(s) => s,
			None => return Poll::Ready(Ok(()))
		};

		// load next bytes
		match stream.poll_bytes(cx) {
			Poll::Ready(Ok(Some(bytes))) => {
				debug_assert!(bytes.len() > 0, "received empty bytes from BytesStream");
				*this.active = bytes;
				Self::fill_buf_from_active(buf, this.active);
				Poll::Ready(Ok(()))
			},
			Poll::Ready(Ok(None)) => {
				self.project().stream.set(None);
				Poll::Ready(Ok(()))
			},
			Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
			Poll::Pending => Poll::Pending
		}

	}
}


#[cfg(test)]
mod tests {

	use super::*;
	use tokio::io::AsyncReadExt;

	#[tokio::test]
	async fn test_stuff() {

		let stream = Bytes::from("some bytes");
		let mut reader = StreamReader::new(stream);

		let mut buf = [0; 5];

		assert_eq!(reader.read_exact(&mut buf[..]).await.unwrap(), 5);
		assert_eq!(&buf, b"some ");
		assert_eq!(reader.read_exact(&mut buf[..]).await.unwrap(), 5);
		assert_eq!(&buf, b"bytes");

		assert_eq!(reader.read(&mut buf[..]).await.unwrap(), 0);
		assert_eq!(&buf, b"bytes");
		assert_eq!(reader.read(&mut buf[..]).await.unwrap(), 0);
		assert_eq!(&buf, b"bytes");

	}

}