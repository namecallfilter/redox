use std::io::Read;

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use flate2::read::GzDecoder;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParserError {
	#[error("Failed to find base64 data")]
	MissingData,
	#[error("Failed to decode base64: {0}")]
	Base64Error(#[from] base64::DecodeError),
	#[error("Failed to decompress gzip: {0}")]
	GzipError(#[from] std::io::Error),
	#[error("Malformed level string")]
	MalformedLevel,
}

pub fn parse_level_data(encoded_data: &str) -> Result<String, ParserError> {
	let clean_data = if let Some(idx) = encoded_data.find("H4sI") {
		&encoded_data[idx..]
	} else {
		encoded_data.trim()
	};

	let decoded_bytes = URL_SAFE.decode(clean_data)?;

	let mut decoder = GzDecoder::new(&decoded_bytes[..]);
	let mut s = String::new();
	decoder.read_to_string(&mut s)?;

	Ok(s)
}

#[derive(Debug)]
pub struct RawObject {
	pub properties: Vec<(String, String)>,
}

pub fn parse_objects(level_string: &str) -> Vec<RawObject> {
	let mut objects = Vec::new();

	for object_str in level_string.split(';') {
		if object_str.trim().is_empty() {
			continue;
		}

		let mut properties = Vec::new();
		let tokens: Vec<&str> = object_str.split(',').collect();

		let mut i = 0;
		while i < tokens.len() {
			if i + 1 >= tokens.len() {
				break;
			}

			let key = tokens[i];
			let val = tokens[i + 1];

			properties.push((key.to_string(), val.to_string()));
			i += 2;
		}

		if !properties.is_empty() {
			objects.push(RawObject { properties });
		}
	}

	objects
}
