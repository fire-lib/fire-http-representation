
macro_rules! enum_method {
	(enum $name:ident {
		$($variant:ident = $val:expr),*
	}) => {

		/// An enum of all possible http methods.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[non_exhaustive]
		pub enum $name {
			$(
				#[doc="`"]
				#[doc=$val]
				#[doc="`"]
				$variant
			),*
			// maybe add other???
		}

		impl $name {
			/// Returns the method as an uppercase string.
			pub fn as_str(&self) -> &'static str {
				match self {
					$(Self::$variant => $val),*
				}
			}
		}

		impl std::convert::TryFrom<&str> for $name {
			type Error = ();

			fn try_from(value: &str) -> Result<Self, Self::Error> {
				match value.to_ascii_uppercase().as_ref() {
					$($val => Ok(Self::$variant)),*,
					_ => Err(())
				}
			}
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{}", self.as_str())
			}
		}

	}
}

// METHOD
enum_method! {
	enum Method {
		Get = "GET",
		Post = "POST",
		Put = "PUT",
		Delete = "DELETE",
		Head = "HEAD",
		Options = "OPTIONS",
		Connect = "CONNECT",
		Patch = "PATCH",
		Trace = "TRACE"
	}
}