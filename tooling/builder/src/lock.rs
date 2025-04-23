use crate::util;
use eks_validator::structs::Metadata;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;


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

// fn check_extensions_against_lock(project_root: &PathBuf, slugs: Vec<String>)
// { 	match read_metadata_lock(project_root) {
// 		Ok(metadata) => {
// 			for slug in slugs {
// 				if metadata.contains_key(&slug) {
// 					println!("✅ Extension '{}' is already in the lock file", slug);
// 				} else {
// 					println!("❌ Extension '{}' is NOT in the lock file", slug);
// 				}
// 			}
// 		}
// 		Err(err) => {
// 			eprintln!("Error loading metadata lock: {}", err);
// 		}
// 	}
// }
