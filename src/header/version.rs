
// Macro to create status codes
macro_rules! enum_version {
	(enum $name:ident {
		$($variant:ident = $major:expr, $minor:expr),*
	}) => {

		/// An enum of the most used http versions.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		pub enum $name {
			$($variant),*
		}

		impl $name {
			/// Returns the major version number.
			pub fn major(&self) -> usize {
				self.full().0
			}

			/// Returns the minor version number.
			pub fn minor(&self) -> usize {
				self.full().1
			}

			/// Returns both the major and minor version number.
			pub fn full(&self) -> (usize, usize) {
				match self {
					$(Self::$variant => ($major, $minor)),*
				}
			}
		}

		use std::cmp::Ordering::{ self, Equal };

		impl std::cmp::PartialOrd for $name {
			fn partial_cmp(&self, other: &Self) -> Option<Ordering> {

				let this = self.full();
				let other = other.full();

				Some(match this.0.cmp(&other.0) {
					Equal => this.1.cmp(&other.1),
					cmp => cmp
				})
			}
		}

		impl std::convert::TryFrom<(usize, usize)> for $name {
			type Error = ();

			fn try_from(value: (usize, usize)) -> Result<Self, Self::Error> {
				match value {
					$(($major, $minor) => Ok(Self::$variant)),*,
					_ => Err(())
				}
			}
		}

		impl std::fmt::Display for $name {
			/// outputs: `HTTP/<major>.<minor>`
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				let full = self.full();
				write!(f, "HTTP/{}.{}", full.0, full.1)
			}
		}
	}
}

// Version
enum_version! {
	enum Version {
		One = 1, 0,
		OnePointOne = 1, 1,
		Two = 2, 0,
		Three = 3, 0
	}
}

impl Default for Version {
	/// At the moment the default is `Version::OnePointOne`.
	fn default() -> Self {
		Self::OnePointOne
	}
}