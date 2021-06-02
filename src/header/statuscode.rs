
// TODO: update doc when rust 1.54 comes out.

// Macro to create status codes
macro_rules! enum_status_code {
	(enum $name:ident {
		$($variant:ident = $num:expr, $str:expr),*
	}) => {

		/// An enum of all possible http status codes.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[repr(u16)]
		#[non_exhaustive]
		pub enum $name {
			$(
				#[doc="`"]
				// not working
				// #[doc=$num]
				#[doc=$str]
				#[doc="`"]
				$variant = $num
			),*
		}

		impl $name {
			/// Returns the corresponding message to the give
			/// `StatusCode`.
			pub fn message(&self) -> &'static str {
				match self {
					$(Self::$variant => $str),*
				}
			}

			/// Returns the corresponding number to the give
			/// `StatusCode`.
			pub fn code(&self) -> u16 {
				*self as u16
			}
		}

		impl std::convert::TryFrom<u16> for $name {
			type Error = ();

			fn try_from(value: u16) -> Result<Self, Self::Error> {
				match value {
					$($num => Ok(Self::$variant)),*,
					_ => Err(())
				}
			}
		}

		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "{} {}", self.code(), self.message())
			}
		}
	}
}



// Status Code Definitions
// Taken from HTTP/1.1 https://tools.ietf.org/html/rfc2616#section-6.1.1

enum_status_code! {
	enum StatusCode {

		// Informational
		Continue = 100, "Continue",
		SwitchingProtocols = 101, "Switching Protocols",

		// Success
		Ok = 200, "OK",
		Created = 201, "Created",
		Accepted = 202, "Accepted",
		NonAuthoritativeInformation = 203, "Non-Authoritative Information",
		NoContent = 204, "No Content",
		ResetContent = 205, "Reset Content",
		PartialContent = 206, "Partial Content",
		
		// Redirection
		MultipleChoices = 300, "Multiple Choices",
		MovedPermanently = 301, "Moved Permanently",
		Found = 302, "Found",
		SeeOther = 303, "See Other",
		NotModified = 304, "Not Modified",
		UseProxy = 305, "Use Proxy",
		TemporaryRedirect = 307, "Temporary Redirect",

		// Client Error
		BadRequest = 400, "Bad Request",
		Unauthorized = 401, "Unauthorized",
		PaymentRequired = 402, "Payment Required",
		Forbidden = 403, "Forbidden",
		NotFound = 404, "Not Found",
		MethodNotAllowed = 405, "Method Not Allowed",
		NotAcceptable = 406, "Not Acceptable",
		ProxyAuthenticationRequired = 407, "Proxy Authentication Required",
		RequestTimeout = 408, "Request Time-out",
		Conflict = 409, "Conflict",
		Gone = 410, "Gone",
		LengthRequired = 411, "Length Required",
		PreconditionFailed = 412, "Precondition Failed",
		RequestEntityTooLarge = 413, "Request Entity Too Large",
		RequestURITooLarge = 414, "Request-URI Too Large",
		UnsupportedMediaType = 415, "Unsupported Media Type",
		RequestedRangeNotSatisfiable = 416, "Requested range not satisfiable",
		ExpectationFailed = 417, "Expectation Failed",

		// Server Error
		InternalServerError = 500, "Internal Server Error",
		NotImplemented = 501, "Not Implemented",
		BadGateway = 502, "Bad Gateway",
		ServiceUnavailable = 503, "Service Unavailable",
		GatewayTimeout = 504, "Gateway Time-out",
		HTTPVersionNotSupported = 505, "HTTP Version not supported"
	}
}