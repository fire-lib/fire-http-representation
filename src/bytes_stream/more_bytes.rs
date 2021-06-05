
use super::{BytesStream, BytesStreamExt};

use std::pin::Pin;
use std::task::{ Context, Poll };
use std::io::Read;
use std::collections::VecDeque;

use bytes::{Bytes, Buf};
use tokio::io::{ self, AsyncWrite, AsyncWriteExt };

/// A structure that holds multiple `Bytes`.
/// 
/// It makes it possible to implement `io::Read` over it
/// to make for example deserialization a bit more memory efficient.
pub struct MoreBytes {
	// this queue never contains empty bytes
	queue: VecDeque<Bytes>
}

impl MoreBytes {

	/// if you call this function
	/// you need to make sure no bytes is empty
	pub(crate) fn new(queue: VecDeque<Bytes>) -> Self {
		Self { queue }
	}

	pub(crate) fn empty() -> Self {
		Self::new(VecDeque::new())
	}

	/// empty bytes won't be appended
	pub(crate) fn push(&mut self, bytes: Bytes) {
		if !bytes.is_empty() {
			self.queue.push_back(bytes);
		}
	}

	/// Returns true if there are no bytes left.
	pub fn is_empty(&self) -> bool {
		self.queue.is_empty()
	}

	/// Returns how many bytes are still remaining.
	/// 
	/// ## Note
	/// Here not the struct `Bytes` are counted but
	/// the actual byte amount.
	pub fn len(&self) -> usize {
		self.queue.iter().map(|b| b.len()).sum::<usize>()
	}

	/// Fills MoreBytes with more bytes from a `BytesStream`.
	pub(crate) async fn fill_from_stream<S>(&mut self, mut stream: S) -> io::Result<()>
	where S: BytesStream + Unpin {
		while let Some(bytes) = stream.next_bytes().await? {
			self.push(bytes);
		}
		Ok(())
	}

	/// Returns the active `Bytes`. Here meaning the first element in the queue.
	fn active(&mut self) -> Option<&mut Bytes> {
		self.queue.get_mut(0)
	}

	fn get_slice(&self) -> Option<&[u8]> {
		self.queue.get(0)
			.map(|b| &**b)
	}

	/// ## Panics
	/// If the queue is empty.
	#[inline]
	fn advance(&mut self, pos: usize) -> usize {
		let act = self.active().unwrap();
		debug_assert!(act.len() >= pos);
		act.advance(pos);
		if act.is_empty() {
			let _ = self.queue.pop_front();
		}

		pos
	}

	/// Returning the next `Bytes` removing it from `MoreBytes`.
	pub fn next_bytes(&mut self) -> Option<Bytes> {
		self.queue.pop_front()
	}

	/// Combines all `Bytes` into one big `Vec`.
	/// 
	/// ## Note
	/// Avoid this if possible, since it's not really Efficient.
	pub fn to_vec(&self) -> Vec<u8> {
		// should calculate capacity
		let mut vec = Vec::with_capacity(self.len());
		for bytes in self.queue.iter() {
			vec.extend_from_slice(bytes);
		}
		vec
	}

	/// Writes all bytes to an `AsyncWrite` implementor.
	pub async fn copy_to_async_write<W>(&self, writer: &mut W) -> io::Result<()>
	where W: AsyncWrite + Unpin {
		for bytes in self.queue.iter() {
			writer.write_all(&*bytes).await?;
		}
		Ok(())
	}

}

impl Read for MoreBytes {
	/// This won't never return an error.
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let slice = match self.get_slice() {
			Some(s) => s,
			None => return Ok(0)
		};

		let len = slice.len()
			.min(buf.len());

		buf[..len].copy_from_slice(&slice[..len]);
		self.advance(len);

		Ok(len)
	}
}

impl BytesStream for MoreBytes {
	/// This will always return `Poll::Ready(Ok(_))`.
	fn poll_bytes(self: Pin<&mut Self>, _: &mut Context) -> Poll<io::Result<Option<Bytes>>> {
		let this = self.get_mut();
		Poll::Ready(Ok(this.next_bytes()))
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_io_read() {

		let mut b = MoreBytes::empty();
		b.push(b"Hello, "[..].into());
		b.push(b"World!"[..].into());

		let mut s = Vec::new();
		let mut buf = [0, 0];

		loop {
			match b.read(&mut buf[..]).unwrap() {
				0 => break,
				read => {
					s.extend_from_slice(&buf[..read]);
				}
			}
		}

		assert_eq!(s, b"Hello, World!");
		assert!(b.next_bytes().is_none());

	}

}