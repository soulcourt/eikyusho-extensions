use crate::lock;
use eks_validator::structs::{Metadata, ServerIndex};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub(crate) fn generate_server_index(
	project_root: &PathBuf,
	all_metadata: &Vec<Metadata>,
	lock: &HashMap<String, HashMap<String, String>>,
) {
	let mut index_list = vec![];

	for metadata in all_metadata {
		let id = match lock::get_lock_entry_id(lock, &metadata.extension.slug) {
			Some(id) => id.to_string(),
			None => {
				log::warn!(
                    "Skipping {} — no ID found in metadata-lock",
                    metadata.extension.slug
                );
				continue;
			}
		};

		let extension =  &metadata.extension;

		let index = ServerIndex {
			id: id.clone(),
			name: extension.name.clone(),
			icon: extension.icon.clone(),
			language: extension.language.clone(),
			version_code: extension.version_code,
			version_name: extension.version_name.clone(),
		};

		index_list.push(index);
	}

	let output_pretty = project_root.join("index.json");
	let output_min = project_root.join("index.min.json");

	create_index_file("📘Prettified",  &output_pretty, &index_list);
	create_index_file("📗Minified", &output_min, &index_list);
}

fn create_index_file(title: &str,  output_min: &PathBuf, index_list: &Vec<ServerIndex>) {
	if let Ok(min_json) = serde_json::to_string(&index_list) {
		if let Err(e) = fs::write(&output_min, min_json) {
			log::error!("Failed to write {title}.json: {}", e);
		} else {
			log::info!("{title} written to {}", output_min.display());
		}
	}
}