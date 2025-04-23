use eks_validator::structs::Metadata;
use std::collections::HashMap;
use std::{fs, path::PathBuf, process::Command};

mod lock;
mod util;

fn main() {
	env_logger::init();

	let project_root = util::get_project_root();
	let extensions_src_dir = project_root.join("src");

	if !extensions_src_dir.exists() {
		util::exit(1, "Extensions source directory does not exist.", true);
	}

	let mut lock = match util::read_metadata_lock(&project_root) {
		Ok(metadata) => metadata,
		Err(_) => {
			util::exit(1, "Error loading metadata lock", true);
		}
	};

	process_languages(&extensions_src_dir, &project_root, &mut lock);

	let stale = lock::check_extensions_against_lock(&lock, &extensions_src_dir);

	if !stale.is_empty() {
		lock::remove_stale_entries_from_lock(&mut lock, &stale);

		if let Err(err) = lock::write_metadata_lock(&project_root, &lock) {
			log::error!("Failed to write metadata-lock.toml: {}", err);
		}
	}
}

fn process_languages(
	extensions_src_dir: &PathBuf,
	project_root: &PathBuf,
	mut lock: &mut HashMap<String, HashMap<String, String>>,
) {
	for lang in fs::read_dir(extensions_src_dir).unwrap() {
		let lang_path = lang.unwrap().path();

		if !lang_path.is_dir() {
			util::exit(
				1,
				"Only directories are allowed in the extensions source directory",
				true,
			);
		}

		process_extensions_in_language(&lang_path, &project_root, &mut lock);
	}
}

fn process_extensions_in_language(
	lang_path: &PathBuf,
	project_root: &PathBuf,
	mut lock: &mut HashMap<String, HashMap<String, String>>,
) {
	for ext_entry in fs::read_dir(&lang_path).unwrap() {
		let ext_path = ext_entry.unwrap().path();

		if !ext_path.is_dir() || !util::has_cargo_toml(&ext_path) {
			continue;
		}

		let metadata = match util::read_and_validate_metadata(&ext_path, &project_root) {
			Some(m) => m,
			None => return,
		};

		if should_build(&metadata, &project_root, &mut lock) {
			build_extension(&ext_path);
		}
	}
}

fn should_build(
	metadata: &Metadata,
	project_root: &PathBuf,
	mut lock: &mut HashMap<String, HashMap<String, String>>,
) -> bool {
	match lock.contains_key(&metadata.extension.slug) {
		true => {
			let build_required = lock::extension_requires_build(&metadata, &lock);

			if build_required {
				lock::update_lock_entry(&mut lock, &metadata);
				persist_lock(&lock, &project_root);
			}

			build_required
		},
		false => {
			lock::add_entry_to_lock(&mut lock, &metadata);
			persist_lock(&lock, &project_root);
			true
		}
	}
}


fn persist_lock(lock: &HashMap<String, HashMap<String, String>>, project_root: &PathBuf) {
	match lock::write_metadata_lock(project_root, lock) {
		Ok(()) => log::info!("Lock file updated!"),
		Err(err) => log::error!("Error updating lock file: {}", err),
	}
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
