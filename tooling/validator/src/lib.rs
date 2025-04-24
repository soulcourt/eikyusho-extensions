use structs::Metadata;

pub mod structs;

pub fn validate_metadata(metadata: &Metadata) -> Result<(), String> {
	let extension = &metadata.extension;

	if extension.slug.trim().is_empty() {
		return Err("Field `slug` is empty.".to_string());
	}

	if extension.name.trim().is_empty() {
		return Err("Field `name` is empty.".to_string());
	}

	if extension.version_name.trim().is_empty() {
		return Err("Field `version_name` is empty.".to_string());
	}

	if extension.language.trim().is_empty() {
		return Err("Field `slug` is empty.".to_string());
	}

	Ok(())
}
