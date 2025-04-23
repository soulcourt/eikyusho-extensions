use crate::util;
use eks_validator::structs::Metadata;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Returns true if the extension needs to be built:
/// - It's not present in the lock file, or
/// - Its version_code is higher than the one in the lock.
pub(crate) fn extension_requires_build(
	metadata: &Metadata,
	lock_data: &HashMap<String, HashMap<String, String>>,
) -> bool {
	match lock_data.get(&metadata.extension.slug) {
		Some(entry) => match entry.get("version_code") {
			Some(locked_vc_str) => {
				metadata.extension.version_code > locked_vc_str.parse::<u32>().unwrap_or(0)
			}
			None => true,
		},
		None => true,
	}
}

pub fn write_metadata_lock(
	project_root: &PathBuf,
	metadata: &HashMap<String, HashMap<String, String>>,
) -> Result<(), String> {
	let lock_path = project_root.join("metadata-lock.toml");

	let toml_str = toml::to_string(metadata).map_err(|e| e.to_string())?;

	fs::write(&lock_path, toml_str).map_err(|e| e.to_string())
}

pub(crate) fn add_entry_to_lock(
	lock: &mut HashMap<String, HashMap<String, String>>,
	metadata: &Metadata,
) {
	if lock.contains_key(&metadata.extension.slug) {
		return;
	}

	log::debug!("🔁 Adding '{}' to metadata lock", metadata.extension.slug);

	let mut entry = HashMap::new();
	entry.insert("id".to_string(), util::generate_uuid());
	entry.insert(
		"version_code".to_string(),
		metadata.extension.version_code.to_string(),
	);

	lock.insert(metadata.extension.slug.clone(), entry);
}

pub(crate) fn update_lock_entry(
	lock: &mut HashMap<String, HashMap<String, String>>,
	metadata: &Metadata,
) {
	if let Some(existing_entry) = lock.get_mut(&metadata.extension.slug) {
		existing_entry.insert(
			"version_code".to_string(),
			metadata.extension.version_code.to_string(),
		);
		log::info!(
            "Updated version_code for '{}' to {}",
            metadata.extension.slug,
            metadata.extension.version_code
        );
	} else {
		log::error!(
            "Cannot update version_code: '{}' not found in metadata lock",
            metadata.extension.slug
        );
	}
}

pub(crate) fn check_extensions_against_lock(
	lock_data: &HashMap<String, HashMap<String, String>>,
	extensions_src_dir: &Path,
) -> Vec<String> {
	let mut stale_entries = Vec::new();

	for slug in lock_data.keys() {
		let parts: Vec<&str> = slug.split('.').collect();

		if parts.len() < 2 {
			log::warn!("Invalid slug format: {}", slug);
			continue;
		}

		let lang = parts[0];
		let ext_name = parts[1];
		let ext_path = extensions_src_dir.join(lang).join(ext_name);

		if !ext_path.exists() {
			stale_entries.push(slug.clone());
		}
	}

	stale_entries
}

pub(crate) fn remove_stale_entries_from_lock(
	lock_data: &mut HashMap<String, HashMap<String, String>>,
	stale_slugs: &[String],
) {
	for slug in stale_slugs {
		lock_data.remove(slug);
		log::info!("Removed stale lock entry: {}", slug);
	}
}