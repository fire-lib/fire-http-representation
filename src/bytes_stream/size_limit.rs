
use super::DEF_CAPACITY;

use std::fmt;

use tokio::io;

/// The error type that is returned when the size limit is reached.
/// 
/// Will mostly be returned in an `io::Error(Kind::Other)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SizeLimitReached(usize);

impl SizeLimitReached {
	/// Returns true if the `io::Error` contains
	/// an `SizeLimitReached` error.
	pub fn is_reached(e: &io::Error) -> bool {
		let dyn_err = match e.get_ref() {
			Some(e) => e,
			None => return false
		};
		dyn_err.is::<Self>()
	}

	/// Downcast an `io::Error` into a `SizeLimitReached`.
	pub fn downcast(e: &io::Error) -> Option<Self> {
		e.get_ref()?
			.downcast_ref()
			.map(Clone::clone)
	}
}

impl fmt::Display for SizeLimitReached {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Debug::fmt(self, f)
	}
}

impl std::error::Error for SizeLimitReached {}

#[derive(Debug, PartialEq, Eq)]
pub struct SizeLimit {
	/// if read > max && max != 0
	/// the limit was surpassed
	/// and an error should be returned
	read: usize,
	// if max == 0
	// no size limit applies
	max: usize
}

impl SizeLimit {

	#[inline]
	pub fn new(max: usize) -> Self {
		assert!(max > 0, "max needs to bigger than zero");
		Self { read: 0, max }
	}

	pub fn empty() -> Self {
		Self { read: 0, max: 0 }
	}

	/// Returns:
	/// no size limit: DEF_CAPCITY
	/// max > read: size
	/// read >= max: 0
	pub fn new_capacity(&self) -> usize {
		if self.max == 0 {
			DEF_CAPACITY
		} else if self.max <= self.read {
			0
		} else {
			(self.max - self.read).min(DEF_CAPACITY)
		}
	}

	// #[allow(dead_code)]// only used with feature = "hyper_body"
	// pub fn max(&self) -> Option<usize> {
	// 	match self.max {
	// 		0 => None,
	// 		m => Some(m)
	// 	}
	// }

	// #[allow(dead_code)]// only used with feature = "hyper_body"
	// pub fn max_reached(&self) -> bool {
	// 	self.max > 0 && self.read >= self.max
	// }

	#[cfg(any(feature = "hyper_body", test))]
	pub fn surpassed(&self) -> bool {
		self.max > 0 && self.read > self.max
	}

	// pub fn add_read(&mut self, read: usize) {
	// 	if self.max != 0 {
	// 		self.read += read;
	// 		debug_assert!(self.read <= self.max);
	// 	}
	// }

	// returns true if max is reached or over
	// #[allow(dead_code)]// only used with feature = "hyper_body"
	// pub fn add_read_clip(&mut self, read: usize) {
	// 	if self.max != 0 {
	// 		let n_read = self.read + read;
	// 		self.read = self.max.min(n_read);
	// 	}
	// }

	/// adds read amount
	/// returns an io error with sizelimit reached
	/// if the limit was surpassed
	pub fn add_read_res(&mut self, read: usize) -> io::Result<()> {
		self.read += read;
		if self.max > 0 && self.read > self.max {
			Err(io::Error::new(
				io::ErrorKind::Other,
				SizeLimitReached(self.max)
			))
		} else {
			Ok(())
		}
	}

	// panics if already read more than max
	// or max == 0
	#[cfg(any(feature = "hyper_body", test))]
	pub fn set(&mut self, max: usize) {
		assert!(max > 0, "max needs to bigger than 0");
		assert!(self.read <= max, "max needs to smaller than already read size");
		self.max = max;
	}

}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn some_tests() {

		let mut limit = SizeLimit::empty();
		assert_eq!(limit.new_capacity(), DEF_CAPACITY);
		// assert_eq!(limit.max(), None);
		// assert!(!limit.max_reached());

		limit.set(2);
		assert_eq!(SizeLimit::new(2), limit);
		assert_eq!(limit.new_capacity(), 2);
		// assert_eq!(limit.max(), Some(2));
		// assert!(!limit.max_reached());

		assert!(limit.add_read_res(2).is_ok());
		// assert!(limit.max_reached());
		assert!(!limit.surpassed());

		assert!(limit.add_read_res(1).is_err());
		// assert!(limit.max_reached());
		assert!(limit.surpassed());
		assert_eq!(limit.new_capacity(), 0);

	}

}