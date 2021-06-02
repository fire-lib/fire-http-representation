
use std::convert::TryInto;

use http as raw;

#[cfg(feature = "encdec")]
fn encode(s: impl AsRef<[u8]>) -> raw::HeaderValue {
	let s: String = percent_encoding::percent_encode(
		s.as_ref(),
		percent_encoding::CONTROLS
	).collect();
	// does not allocate again
	let b: bytes::Bytes = s.into();
	// now lets make a header value
	// TODO probably can be changed to
	// from maybe shared unchecked
	// but need to check first control covers all cases
	raw::HeaderValue::from_maybe_shared(b).unwrap()
}

/// Converts a value into a `HeaderValue`.
/// 
/// Some types can not be converted for sure, like
/// string's and bytes. Numbers (except floats) cannot fail.
pub trait IntoHeaderValue {
	/// Tries to convert to a `HeaderValue`.
	fn try_into_header_value(self) -> Option<raw::HeaderValue>;

	#[cfg(feature = "encdec")]
	fn into_enc_header_value(self) -> raw::HeaderValue;
}

macro_rules! impl_into_header_value {
	($(
		$s:ty,
			$self1:ident => $ex1:expr,
			$self2:ident => $ex2:expr
	),*) => ($(
		impl IntoHeaderValue for $s {
			#[inline]
			fn try_into_header_value($self1) -> Option<raw::HeaderValue> {
				$ex1
			}

			#[cfg(feature = "encdec")]
			#[inline]
			fn into_enc_header_value($self2) -> raw::HeaderValue {
				$ex2
			}
		}
	)*);
	(REF, $(
		$s:ty,
			$self1:ident => $ex1:expr,
			$self2:ident => $ex2:expr
	),*) => ($(
		impl<'a> IntoHeaderValue for &'a $s {
			#[inline]
			fn try_into_header_value($self1) -> Option<raw::HeaderValue> {
				$ex1
			}

			#[cfg(feature = "encdec")]
			#[inline]
			fn into_enc_header_value($self2) -> raw::HeaderValue {
				$ex2
			}
		}
	)*);
}

impl_into_header_value!{
	raw::header::HeaderName,
		self => Some(self.into()),
		self => self.into(),
	i16,
		self => Some(self.into()),
		self => self.into(),
	i32,
		self => Some(self.into()),
		self => self.into(),
	i64,
		self => Some(self.into()),
		self => self.into(),
	isize,
		self => Some(self.into()),
		self => self.into(),
	u16,
		self => Some(self.into()),
		self => self.into(),
	u32,
		self => Some(self.into()),
		self => self.into(),
	u64,
		self => Some(self.into()),
		self => self.into(),
	usize,
		self => Some(self.into()),
		self => self.into(),
	String,
		self => self.try_into().ok(),
		self => encode(self),
	Vec<u8>,
		self => self.try_into().ok(),
		self => encode(self)
}

impl_into_header_value!{ REF,
	raw::HeaderValue,
		self => Some(self.into()),
		self => self.clone(),
	[u8],
		self => self.try_into().ok(),
		self => encode(self),
	str,
		self => self.try_into().ok(),
		self => encode(self)
}