use eks_validator::structs::Metadata;
use figment::providers::Toml;
use figment::{
	Figment,
	providers::{Format, Yaml},
};
use std::collections::HashMap;
use std::{env, path::PathBuf};
use uuid::Uuid;
use eks_validator::validate_metadata;
use crate::util;

pub(crate) fn get_project_root() -> PathBuf {
	env::current_dir()
		.unwrap()
		.parent()
		.and_then(|p| p.parent())
		.map(|p| p.to_path_buf())
		.unwrap_or_else(|| {
			log::warn!("Failed to find project root.");
			std::process::exit(1);
		})
}

pub(crate) fn has_cargo_toml(path: &PathBuf) -> bool {
	path.join("Cargo.toml").exists()
}

pub(crate) fn read_metadata(path: &PathBuf) -> Result<Metadata, String> {
	let metadata_path = path.join("res/metadata.yaml");

	let metadata: Result<Metadata, _> = Figment::new().merge(Yaml::file(metadata_path)).extract();

	match metadata {
		Ok(metadata) => Ok(metadata),
		Err(_) => Err("Failed to parse the YAML content.".to_string()),
	}
}

pub(crate) fn read_metadata_lock(
	project_root: &PathBuf,
) -> Result<HashMap<String, HashMap<String, String>>, String> {
	let lock_path = project_root.join("metadata-lock.toml");

	let lock: Result<HashMap<String, HashMap<String, String>>, _> =
		Figment::new().join(Toml::file(&lock_path)).extract();

	match lock {
		Ok(metadata) => Ok(metadata),
		Err(e) => Err(format!("Failed to parse the TOML content: {}", e)),
	}
}

pub(crate) fn read_and_validate_metadata(ext_path: &PathBuf, project_root: &PathBuf) -> Option<Metadata> {
	let display_path = ext_path.strip_prefix(&project_root.join("src")).unwrap();

	let metadata = match util::read_metadata(ext_path) {
		Ok(m) => m,
		Err(err) => {
			log::warn!("Skipping {:?}: {}", display_path, err);
			return None;
		}
	};

	if let Err(err) = validate_metadata(&metadata) {
		log::warn!("Skipping {:?}: {}", display_path, err);
		return None;
	}

	Some(metadata)
}

pub(crate) fn generate_uuid() -> String {
	Uuid::new_v4().to_string()
}

pub(crate) fn exit(code: i32, msg: &str, error: bool) -> ! {
	if error {
		log::error!("{}", msg);
	} else {
		log::info!("{}", msg);
	}
	std::process::exit(code);
}
