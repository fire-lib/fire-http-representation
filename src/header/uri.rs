
use http as raw;

// TODO add query str parser.
// TODO add segments probably
// TODO maybe there is a way to substract a part from an uri.
// making it possible to parse it more easely in a route.

/// Contains a request uri.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Uri(raw::Uri);

impl Uri {

	//  This is not documented but won't break between minor changes.
	/// Returns None if no scheme or host was found.
	#[doc(hidden)]
	pub fn new(raw: raw::Uri) -> Option<Self> {
		let _ = raw.scheme_str()?;
		let _ = raw.host()?;
		Some(Self(raw))
	}

	/// Returns the used scheme.
	pub fn scheme(&self) -> &str {
		self.0.scheme_str().unwrap()
	}

	/// Returns true if the used scheme is https.
	pub fn is_https(&self) -> bool {
		self.scheme() == "https"
	}

	/// Returns true if the used scheme is http.
	pub fn is_http(&self) -> bool {
		self.scheme() == "http"
	}

	/// Returns the host.
	pub fn host(&self) -> &str {
		self.0.host().unwrap()
	}

	/// Returns the used port if any.
	pub fn port(&self) -> Option<u16> {
		self.0.port_u16()
	}

	/// Returns the path.
	pub fn path(&self) -> &str {
		self.0.path()
	}

	/// Returns the query string.
	pub fn query_str(&self) -> Option<&str> {
		self.0.query()
	}

}