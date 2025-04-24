use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Clone)]
pub struct Metadata {
	pub extension: Extension,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Extension {
	pub slug: String,
	pub name: String,
	pub icon: String,
	pub version_code: u32,
	pub version_name: String,
	pub language: String,
	pub description: String,
}

#[derive(Serialize)]
pub struct ServerIndex {
	pub id: String,
	pub name: String,
	pub icon: String,
	pub language: String,
	pub version_code: u32,
	pub version_name: String,
}