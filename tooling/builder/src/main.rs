use std::{fs, path::PathBuf, process::Command};

mod structs;
mod util;

fn main() {
	env_logger::init();

	let project_root = util::get_project_root();
	let extensions_src_dir = project_root.join("src");

	if !extensions_src_dir.exists() {
		util::exit(1, "Extensions source directory does not exist.", true);
	}

	for lang in fs::read_dir(extensions_src_dir).unwrap() {
		let lang_path = lang.unwrap().path();

		if !lang_path.is_dir() {
			util::exit(
				1,
				"Only directories are allowed in the extensions source directory",
				true,
			);
		}

		for ext_entry in fs::read_dir(&lang_path).unwrap() {
			let ext_path = ext_entry.unwrap().path();

			if !ext_path.is_dir() || !util::has_cargo_toml(&ext_path) {
				continue;
			}

			match util::read_metadata(&ext_path) {
				Ok(_) => {

				}
				Err(err) => {
					let display_path = ext_path.strip_prefix(&project_root.join("src")).unwrap();
					log::warn!(
						"Skipping {:?}: {}",
						display_path,
						err
					);
					continue;
				}
			}

			build_extension(&ext_path);
		}
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
