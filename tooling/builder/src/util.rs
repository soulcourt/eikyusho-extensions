use figment::{
	Figment,
	providers::{Format, Yaml},
};
use std::{env, path::PathBuf};
use eks_validator::{structs::Metadata};

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

	let metadata: Result<Metadata, _> = Figment::new()
		.merge(Yaml::file(metadata_path))
		.extract();

	match metadata {
		Ok(metadata) => Ok(metadata),
		Err(_) => Err("Failed to parse the YAML content.".to_string()),
	}
}

pub(crate) fn exit(code: i32, msg: &str, error: bool) -> ! {
	if error {
		log::error!("{}", msg);
	} else {
		log::info!("{}", msg);
	}
	std::process::exit(code);
}
