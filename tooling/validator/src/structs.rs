use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Metadata {
	pub extension: Extension,
}

#[derive(Debug, Deserialize)]
pub struct Extension {
	pub slug: String,
	pub name: String,
	pub version: String,
	pub language: String,
	pub description: String,
}