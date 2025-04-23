use std::{fs, path::PathBuf, process::Command};
use eks_validator::{validate_metadata};
use eks_validator::structs::Metadata;

mod util;

fn main() {
	env_logger::init();

	let project_root = util::get_project_root();
	let extensions_src_dir = project_root.join("src");

	if !extensions_src_dir.exists() {
		util::exit(1, "Extensions source directory does not exist.", true);
	}

	process_languages(&extensions_src_dir, &project_root);
}

fn process_languages(extensions_src_dir: &PathBuf, project_root: &PathBuf) {
	for lang in fs::read_dir(extensions_src_dir).unwrap() {
		let lang_path = lang.unwrap().path();

		if !lang_path.is_dir() {
			util::exit(
				1,
				"Only directories are allowed in the extensions source directory",
				true,
			);
		}

		process_extensions_in_language(&lang_path, &project_root);
	}
}

fn process_extensions_in_language(lang_path: &PathBuf, project_root: &PathBuf) {
	for ext_entry in fs::read_dir(&lang_path).unwrap() {
		let ext_path = ext_entry.unwrap().path();

		if !ext_path.is_dir() || !util::has_cargo_toml(&ext_path) {
			continue;
		}

		if let Some(_) = read_and_validate_metadata(&ext_path, &project_root) {
			build_extension(&ext_path);
		}
	}
}

fn read_and_validate_metadata(ext_path: &PathBuf, project_root: &PathBuf) -> Option<Metadata> {
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

fn build_extension(extension_path: &PathBuf) {
	let status = Command::new("cargo")
		.arg("+nightly")
		.arg("build")
		.arg("--release")
		.arg("--target")
		.arg("wasm32-unknown-unknown")
		.current_dir(extension_path)
		.status()
		.expect("Failed to run cargo build");

	if status.success() {
		log::info!("Built {:?}", extension_path);
	} else {
		log::error!("Build failed for {:?}", extension_path);
	}
}
