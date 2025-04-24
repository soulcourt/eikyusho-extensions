use eks_validator::structs::Metadata;
use std::collections::HashMap;
use std::{fs, path::PathBuf, process::Command};

mod lock;
mod util;
mod binary;
mod server;

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

	let mut all_metadata = vec![];

	process_languages(&extensions_src_dir, &project_root, &mut lock, &mut all_metadata);

	let stale = lock::check_extensions_against_lock(&lock, &extensions_src_dir);

	if !stale.is_empty() {
		lock::remove_stale_entries_from_lock(&project_root, &mut lock, &stale);
	}

	server::generate_server_index(&project_root, &all_metadata, &lock);
}

fn process_languages(
	extensions_src_dir: &PathBuf,
	project_root: &PathBuf,
	mut lock: &mut HashMap<String, HashMap<String, String>>,
	all_metadata: &mut Vec<Metadata>,
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

		process_extensions_in_language(&lang_path, &project_root, &mut lock, all_metadata);
	}
}

fn process_extensions_in_language(
	lang_path: &PathBuf,
	project_root: &PathBuf,
	mut lock: &mut HashMap<String, HashMap<String, String>>,
	all_metadata: &mut Vec<Metadata>,
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

		all_metadata.push(metadata.clone());

		if should_build(&metadata, &project_root, &mut lock) {
			let extension_built = build_extension(&ext_path);
			if extension_built {
				binary::generate_extension_binary(
					&ext_path,
					&metadata.extension.slug,
					&project_root,
					&lock,
				);
			}
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
				lock::persist_lock(&lock, &project_root);
			}

			build_required
		},
		false => {
			lock::add_entry_to_lock(&mut lock, &metadata);
			lock::persist_lock(&lock, &project_root);
			true
		}
	}
}




fn build_extension(extension_path: &PathBuf) ->  bool {
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
		log::debug!("Built {:?}", extension_path);
		true
	} else {
		log::error!("Build failed for {:?}", extension_path);
		false
	}
}

