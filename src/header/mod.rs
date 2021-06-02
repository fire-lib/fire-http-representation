
use std::net::SocketAddr;

#[cfg(feature = "encdec")]
use std::borrow::Cow;

use http as raw;

mod statuscode;
pub use statuscode::StatusCode;

mod method;
pub use method::Method;

mod version;
pub use version::Version;

mod uri;
pub use uri::Uri;

mod contenttype;
pub use contenttype::{ContentType, Mime, AnyMime, Charset};

mod into_header_value;
pub use into_header_value::IntoHeaderValue;

// TODO: add url encoding to the http header values
// allowing to insert any utf8 string

/// Contains all http header values.
/// 
/// This is really similar to `http::header::HeaderMap` except
/// that is uses IntoHeaderValue for inserting. And it does not allow
/// multiples values for a given key.
#[derive(Debug, Clone)]
pub struct HeaderValues(raw::HeaderMap<raw::HeaderValue>);

impl HeaderValues {

	/// Creates a new empty `HeaderValues`.
	pub fn new() -> Self {
		Self(raw::HeaderMap::new())
	}

	//  This is not documented but won't break between minor changes.
	/// Creates a new `HeaderValues` from it's inner type.
	#[doc(hidden)]
	pub fn from_inner(raw: raw::HeaderMap<raw::HeaderValue>) -> Self {
		Self(raw)
	}

	/// Insert a new key and value into the header.
	/// 
	/// If a value to this key is already present
	/// that value is dropped.
	/// 
	/// ## Panics
	/// If the value is not a valid HeaderValue.
	pub fn insert<K, V>(&mut self, key: K, val: V)
	where
		K: raw::header::IntoHeaderName,
		V: IntoHeaderValue
	{
		let val = val.try_into_header_value()
			.expect("invalid HeaderValue");
		let _ = self.0.insert(key, val);
	}

	/// Insert a new key and value into the header. Returning
	/// None if the value is not valid.
	/// 
	/// If a value to this key is already present
	/// that value is dropped.
	pub fn try_insert<K, V>(&mut self, key: K, val: V) -> Option<()>
	where
		K: raw::header::IntoHeaderName,
		V: IntoHeaderValue {
		let _ = self.0.insert(key, val.try_into_header_value()?);
		Some(())
	}

	/// Insert a new key and value into the header. Percent encoding
	/// the value if necessary.
	#[cfg(feature = "encdec")]
	pub fn encode<K, V>(&mut self, key: K, val: V)
	where
		K: raw::header::IntoHeaderName,
		V: IntoHeaderValue
	{
		let val = val.into_enc_header_value();
		let _ = self.0.insert(key, val);
	}

	/// Returns the value if it exists.
	pub fn get<K>(&self, key: K) -> Option<&raw::HeaderValue>
	where K: raw::header::AsHeaderName {
		self.0.get(key)
	}

	/// Returns the value mutably if it exists.
	pub fn get_mut<K>(&mut self, key: K) -> Option<&mut raw::HeaderValue>
	where K: raw::header::AsHeaderName {
		self.0.get_mut(key)
	}

	/// Returns the value as a string if it exists and is valid.
	pub fn get_str<K>(&self, key: K) -> Option<&str>
	where K: raw::header::AsHeaderName {
		self.get(key)
			.and_then(|v| {
				v.to_str().ok()
			})
	}

	/// Returns the value percent decoded as a string if it exists and is valid.
	#[cfg(feature = "encdec")]
	pub fn decode<K>(&self, key: K) -> Option<Cow<'_, str>>
	where K: raw::header::AsHeaderName {
		self.get(key)
			.and_then(|v| {
				percent_encoding::percent_decode(v.as_bytes())
					.decode_utf8()
					.ok()
			})
	}

	/// Insert a new key and a serializeable value. The value will be serialized
	/// as json and percent encoded.
	/// 
	/// Returns `None` if the value could not be serialized or inserted.
	#[cfg(all(feature = "json", feature = "encdec"))]
	pub fn serialize<K, V: ?Sized>(&mut self, key: K, val: &V) -> Option<()>
	where
		K: raw::header::IntoHeaderName,
		V: serde::Serialize
	{
		let v = serde_json::to_string(val).ok()?;
		Some(self.encode(key, v))
	}

	/// Deserializes a given value. Returning `None` if the value
	/// does not exist or is not valid json.
	#[cfg(all(feature = "json", feature = "encdec"))]
	pub fn deserialize<K, D>(&self, key: K) -> Option<D>
	where
		K: raw::header::AsHeaderName,
		D: serde::de::DeserializeOwned {
		let v = self.decode(key)?;
		serde_json::from_str(v.as_ref()).ok()
	}

	//  This is not documented but won't break between minor changes.
	/// Returns the inner `HeaderMap`.
	#[doc(hidden)]
	pub fn into_inner(self) -> raw::HeaderMap<raw::HeaderValue> {
		self.0
	}

}

/// RequestHeader received from a client.
#[derive(Debug, Clone)]
pub struct RequestHeader {
	pub address: SocketAddr,
	pub method: Method,
	pub uri: Uri,
	pub version: Version,
	pub values: HeaderValues
}

impl RequestHeader {

	/// Returns the ip address of the requesting client.
	pub fn address(&self) -> &SocketAddr {
		&self.address
	}

	/// Returns the requesting method.
	pub fn method(&self) -> &Method {
		&self.method
	}

	/// Returns the requesting uri.
	pub fn uri(&self) -> &Uri {
		&self.uri
	}

	/// Returns the http version used for the request.
	pub fn version(&self) -> &Version {
		&self.version
	}

	/// Returns all header values.
	pub fn values(&self) -> &HeaderValues {
		&self.values
	}

	/// Returns a header value from it's key
	/// if it exists and is valid ascii.
	/// 
	/// ## Note
	/// If you wan't a decoded value use `self.values().decode(key)`.
	pub fn value<K>(&self, key: K) -> Option<&str>
	where K: raw::header::AsHeaderName {
		self.values.get_str(key)
	}

}

/// ResponseHeader created from a server.
/// 
/// To create a ResponseHeader you should probably
/// use ResponseHeaderBuilder.
#[derive(Debug, Clone)]
pub struct ResponseHeader {
	pub version: Version,
	pub status_code: StatusCode,
	pub content_type: ContentType,
	pub values: HeaderValues
}

impl ResponseHeader {

	/// Returns the http version.
	pub fn version(&self) -> &Version {
		&self.version
	}

	/// Returns the used status code.
	pub fn status_code(&self) -> &StatusCode {
		&self.status_code
	}

	/// Returns the used content type.
	pub fn content_type(&self) -> &ContentType {
		&self.content_type
	}

	/// Returns all header values.
	pub fn values(&self) -> &HeaderValues {
		&self.values
	}

	/// Returns a header value from it's key
	/// if it exists and is valid ascii.
	/// 
	/// ## Note
	/// If you wan't a decoded value use `self.values().decode(key)`.
	pub fn value<K>(&self, key: K) -> Option<&str>
	where K: raw::header::AsHeaderName {
		self.values.get_str(key)
	}

}

impl Default for ResponseHeader {
	fn default() -> Self {
		Self {
			version: Version::default(),
			status_code: StatusCode::Ok,
			content_type: Mime::Text.into(),
			values: HeaderValues::new()
		}
	}
}

// TODO: can we remove the version???

/// A build to create a `ResponseHeader`.
#[derive(Debug, Clone)]
pub struct ResponseHeaderBuilder {
	pub version: Option<Version>,
	pub status_code: Option<StatusCode>,
	pub content_type: Option<ContentType>,
	pub values: HeaderValues
}

impl ResponseHeaderBuilder {

	/// Creates a new builder.
	pub fn new() -> Self {
		Self {
			version: None,
			status_code: None,
			content_type: None,
			values: HeaderValues::new()
		}
	}

	/// Sets the http version.
	pub fn version(&mut self, version: Version) {
		self.version = Some(version);
	}

	/// Sets the status code.
	pub fn status_code(&mut self, status_code: StatusCode) {
		self.status_code = Some(status_code);
	}

	/// Sets the content type.
	pub fn content_type(&mut self, content_type: impl Into<ContentType>) {
		self.content_type = Some(content_type.into());
	}

	// /// Sets a header value.
	// /// 
	// /// ## Panics
	// /// If the value is not a valid `HeaderValue`.
	// pub fn header<K, V>(&mut self, key: K, val: V)
	// where
	// 	K: raw::header::IntoHeaderName,
	// 	V: IntoHeaderValue {
	// 	self.values.insert(key, val);
	// }

	/// Returns `HeaderValues` mutably.
	pub fn values_mut(&mut self) -> &mut HeaderValues {
		&mut self.values
	}

	/// Builds a ResponseHeader. Using default values for all
	/// not configered fields.
	pub fn build(self) -> ResponseHeader {
		ResponseHeader {
			version: self.version.unwrap_or(Version::default()),
			status_code: self.status_code.unwrap_or(StatusCode::Ok),
			content_type: self.content_type.unwrap_or(ContentType::empty()),
			values: self.values
		}
	}

}


#[cfg(test)]
mod tests {
	#![allow(unused_imports)]

	use super::*;
	use serde::{Serialize, Deserialize};

	#[cfg(feature = "encdec")]
	#[test]
	fn test_encdec() {

		let mut values = HeaderValues::new();
		values.encode("Rocket", "🚀 Rocket");
		let s = values.get_str("Rocket").unwrap();
		assert_eq!(s, "%F0%9F%9A%80 Rocket");

		let s = values.decode("Rocket").unwrap();
		assert_eq!(s, "🚀 Rocket");

	}

	#[cfg(all(feature="encdec", feature="json"))]
	#[test]
	fn test_serde() {

		#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
		struct Value {
			text: String,
			number: usize
		}

		let mut values = HeaderValues::new();
		let val = Value {
			text: "🚀 Rocket".into(),
			number: 42
		};
		values.serialize("Value", &val).unwrap();

		let s = values.get_str("Value").unwrap();
		assert_eq!(s, "{\"text\":\"%F0%9F%9A%80 Rocket\",\"number\":42}");

		let n_val: Value = values.deserialize("Value").unwrap();
		assert_eq!(n_val, val);

	}

}