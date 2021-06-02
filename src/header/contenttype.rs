//! Types related to the `ContentType` http header.
//!
//! ## Note
//! At the moment these are more useful when creating
//! then when parsing a content type.
//!
//! ## Todo
//! At some point this should probably be MediaType
//! and be more granular to be able to parse it more easely.

use std::fmt;
use std::default::Default;

// TODO maybe try to optimize main_type()

// TODO: add more complete doc comment to mime variant
// when 1.54 gets released, See
// https://github.com/rust-lang/rust/issues/78835

// Macro to create status codes
macro_rules! mime {
	(
		$(
			$(#[$variant_attr:meta])*
			$variant:ident = $ext:expr, $mime:expr
		),*
	) => {

		/// A list of the most important mime types.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[non_exhaustive]
		pub enum Mime {
			$(
				$(#[$variant_attr])*
				#[doc = "File extension: `"]
				#[doc = $ext]
				#[doc = "`, mime type: `"]
				#[doc = $mime]
				#[doc = "`"]
				$variant
			),*
		}

		impl Mime {

			/// Returns the file extension.
			/// 
			/// ## Note
			/// To see what mime type is returned see the variant
			/// documentation.
			pub fn ext(&self) -> &'static str {
				match self {
					$(Self::$variant => $ext),*
				}
			}

			/// Returns the mime type.
			/// 
			/// ## Note
			/// To see what mime type is returned see the variant
			/// documentation.
			pub fn mime(&self) -> &'static str {
				match self {
					$(Self::$variant => $mime),*
				}
			}

			/// Returns the type.
			/// 
			/// ## Example
			/// ```
			/// # use fire_http_representation::header::Mime;
			/// let mime = Mime::Jar;
			/// assert_eq!(mime.main_type(), "application");
			/// ```
			pub fn main_type(&self) -> &'static str {
				self.mime().split('/').next().unwrap()
			}

			/// Returns the subtype.
			/// 
			/// ## Example
			/// ```
			/// # use fire_http_representation::header::Mime;
			/// let mime = Mime::Jar;
			/// assert_eq!(mime.sub_type(), "java-archive");
			/// ```
			pub fn sub_type(&self) -> &'static str {
				self.mime().split('/').nth(1).unwrap()
			}

			/// Tries to return a mime type from a file extension.
			pub fn try_from_ext(value: &str) -> Option<Self> {
				match value {
					$($ext => Some(Self::$variant)),*,
					_ => None
				}
			}

			/// Returns a mime type to a given file extension,
			/// if the file extension is unknown `Mime::Binary` is returned.
			pub fn from_ext(value: &str) -> Self {
				Self::try_from_ext(value)
					.unwrap_or_default()
			}

			/// Tries to return a mime type from a mime type string.
			pub fn try_from_mime(value: &str) -> Option<Self> {
				match value {
					$($mime => Some(Self::$variant)),*,
					_ => None
				}
			}

		}

		impl From<&str> for Mime {
			fn from(s: &str) -> Self {
				Self::from_ext(s)
			}
		}

	}
}

mime! {
	// text
	Text = "txt", "text/plain",
	Html = "html", "text/html",
	Js = "js", "application/javascript",
	Css = "css", "text/css",
	Json = "json", "application/json",
	Csv = "csv", "text/csv",
	Doc = "doc", "application/msword",
	Docx = "docx", "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
	Pdf = "pdf", "application/pdf",
	Php = "php", "application/php",
	Rtf = "rtf", "application/rtf",
	Sh = "sh", "application/x-sh",
	Vsd = "vsd", "application/vnd.visio",
	Xml = "xml", "text/xml",

	// imgs
	Jpg = "jpg", "image/jpeg",
	Png = "png", "image/png",
	Gif = "gif", "image/gif",
	Svg = "svg", "image/svg+xml",
	Ico = "ico", "image/vnd.microsoft.icon",
	Tiff = "tiff", "image/tiff",
	Webp = "webp", "image/webp",

	// fonts
	Eot = "eot", "application/vnd.ms-fontobject",
	Ttf = "ttf", "font/ttf",
	Woff = "woff", "font/woff",
	Woff2 = "woff2", "font/woff2",

	// video
	Avi = "avi", "video/x-msvideo",
	Ogv = "ogv", "video/ogg",
	Webm = "webm", "video/webm",
	Mp4 = "mp4", "video/mp4",

	// audio
	Aac = "aac", "audio/aac",
	Mp3 = "mp3", "audio/mpeg",
	Oga = "oga", "audio/ogg",
	Wav = "wav", "audio/wav",
	Weba = "weba", "audio/webm",

	// Archives
	Rar = "rar", "application/vnd.rar",
	Tar = "tar", "application/x-tar",
	Zip = "zip", "application/zip",
	_7Zip = "7z", "application/x-7z-compressed",

	// Binary
	Jar = "jar", "application/java-archive",
	Binary = "bin", "application/octet-stream",
	Wasm = "wasm", "application/wasm"
}

impl Default for Mime {
	fn default() -> Self {
		Self::Binary
	}
}

/// Holds a `Mime` or a `String` containing a mime type.
/// 
/// ## Example
/// ```
/// # use fire_http_representation::header::{Mime, AnyMime, Charset};
/// let known_mime = AnyMime::from(Mime::Js);
/// assert_eq!(known_mime.mime(), "application/javascript");
/// assert_eq!(known_mime.charset().unwrap(), Charset::Utf8);
/// 
/// let unknown_mime = AnyMime::from("application/rust".to_string());
/// assert_eq!(unknown_mime.mime(), "application/rust");
/// assert!(unknown_mime.charset().is_none());
/// 
/// let empty = AnyMime::None;
/// assert_eq!(empty.mime(), "");
/// assert!(empty.charset().is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyMime {
	Known(Mime),
	Unknown(String),
	None
}

impl AnyMime {

	/// Returns the mime type as a string.
	pub fn mime(&self) -> &str {
		match self {
			Self::Known(mime) => mime.mime(),
			Self::Unknown(mime) => &mime,
			Self::None => ""
		}
	}

	/// Returns the mime type as an owned string.
	pub fn to_string(&self) -> String {
		match self {
			Self::Known(mime) => mime.mime().to_string(),
			Self::Unknown(s) => s.clone(),
			Self::None => String::new()
		}
	}

	/// Returns `Charset::Utf8` if the mime type is known and is a text file.
	pub fn charset(&self) -> Option<Charset> {
		match self {
			Self::Known(mime)
				if mime.main_type() == "text" => Some(Charset::Utf8),
			Self::Known(Mime::Js) |
			Self::Known(Mime::Json) => Some(Charset::Utf8),
			_ => None
		}
	}

}

impl fmt::Display for AnyMime {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.mime().fmt(f)
	}
}

impl From<Mime> for AnyMime {
	fn from(mime: Mime) -> Self {
		Self::Known(mime)
	}
}

impl From<String> for AnyMime {
	fn from(value: String) -> Self {
		Self::Unknown(value)
	}
}

/// Http `ContentType` header combining mime and charset.
/// 
/// ## Note
/// The directive `boundary` is not supported.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentType {
	pub mime: AnyMime,
	pub charset: Option<Charset>
}

impl ContentType {

	/// Creates a new `ContentType` and automatically
	/// adding a charset if the mime type is text.
	pub fn new<M>(mime: M) -> Self
	where M: Into<AnyMime> {
		mime.into().into()
	}

	/// Creates a new `ContentType` with a charset
	/// that does not get autmatically set by `AnyMime::charset`.
	pub fn with_charset<M>(mime: M, charset: Charset) -> Self
	where M: Into<AnyMime> {
		Self {
			mime: mime.into(),
			charset: Some(charset)
		}
	}

	/// Creates an empty `ContentType`.
	pub fn empty() -> Self {
		Self {
			mime: AnyMime::None,
			charset: None
		}
	}

	/// Creates a string containing the mimetype and
	/// the charset if available.
	/// 
	/// ## Example
	/// ```
	/// # use fire_http_representation::header::{Mime, ContentType};
	/// let ctn_type = ContentType::new(Mime::Js);
	/// assert_eq!(ctn_type.to_string(), "application/javascript; charset=utf-8");
	/// ```
	pub fn to_string(&self) -> String {
		match &self.charset {
			Some(s) => format!("{}; charset={}", self.mime, s),
			None => self.mime.to_string()
		}
	}

}

impl fmt::Display for ContentType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.to_string().fmt(f)
	}
}

impl<T> From<T> for ContentType
where T: Into<AnyMime> {
	fn from(mime: T) -> Self {
		let mime = mime.into();
		let charset = mime.charset();

		Self { mime, charset }
	}
}



macro_rules! charset {
	($($name:ident = $val:expr),*) => (
		/// A list of charset's that can be used.
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		#[non_exhaustive]
		pub enum Charset {
			$(
				#[doc="str value: `"]
				#[doc=$val]
				#[doc="`"]
				$name
			),*
		}

		impl Charset {

			/// Tries to get a `Charset` from a string.
			pub fn from_str(v: &str) -> Option<Self> {
				match v {
					$($val => Some(Self::$name)),*,
					_ => None
				}
			}

			/// Returns the string representation. See the variant
			/// documentation to see what string is used.
			pub fn as_str(&self) -> &'static str {
				match self {
					$(Self::$name => $val),*
				}
			}
		}

		impl fmt::Display for Charset {
			fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				self.as_str().fmt(f)
			}
		}

		// TODO maybe implement TryFrom
	)
}

charset!{
	Utf8 = "utf-8"
}

impl Default for Charset {
	fn default() -> Self {
		Self::Utf8
	}
}