
use super::BytesStream;

use std::pin::Pin;
use std::task::{ Context, Poll };

use bytes::Bytes;
use tokio::io;

use pin_project_lite::pin_project;

use http_body::Body;
use http::HeaderMap;

pin_project!{
	/// A wrapper around `BytesStream` implementing `http_body::Body`.
	#[derive(Debug)]
	pub struct StreamHttpBody<S> {
		#[pin]
		stream: Option<S>
	}
}

impl<S: BytesStream> StreamHttpBody<S> {
	pub(crate) fn new(stream: Option<S>) -> Self {
		Self { stream }
	}
}


impl<S: BytesStream> Body for StreamHttpBody<S> {
	type Data = Bytes;
	type Error = io::Error;

	fn poll_data(
		self: Pin<&mut Self>,
		cx: &mut Context<'_>
	) -> Poll<Option<io::Result<Self::Data>>> {
		match self.project().stream.as_pin_mut() {
			Some(s) => s,
			None => return Poll::Ready(None)
		}
			.poll_bytes(cx)
			.map(|r| r.transpose())
	}

	fn poll_trailers(
		self: Pin<&mut Self>,
		_: &mut Context<'_>
	) -> Poll<io::Result<Option<HeaderMap>>> {
		Poll::Ready(Ok(None))
	}

	fn is_end_stream(&self) -> bool {
		self.stream.is_none()
	}
}