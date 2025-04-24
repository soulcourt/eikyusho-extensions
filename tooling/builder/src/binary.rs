use crate::lock;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn generate_extension_binary(
	extension_path: &PathBuf,
	metadata_slug: &str,
	project_root: &Path,
	lock: &HashMap<String, HashMap<String, String>>,
) {
	let target_dir = extension_path.join("target/wasm32-unknown-unknown/release");
	let payload_dir = target_dir.join("Payload");

	let id = match lock::get_lock_entry_id(lock, metadata_slug) {
		Some(id) => id,
		None => {
			log::error!("No ID found for slug {} in metadata-lock", metadata_slug);
			return;
		}
	};

	let output_eks = project_root
		.join("extensions")
		.join(&id)
		.join("extension.eks");

	create_directories(&payload_dir, &output_eks);

	copy_resources(extension_path, &payload_dir);

	copy_wasm(&payload_dir, &target_dir);

	copy_icon(extension_path, project_root, metadata_slug, &id);

	let zip_status = Command::new("zip")
		.arg("-qr")
		.arg(&output_eks)
		.arg("Payload")
		.current_dir(&target_dir)
		.status()
		.expect("Failed to run zip command");

	if zip_status.success() {
		log::info!("Created {}", output_eks.display());
	} else {
		log::error!("Failed to zip {}", output_eks.display());
	}
}

fn create_directories(payload_dir: &PathBuf, output_eks: &Path) {
	if let Some(parent) = output_eks.parent() {
		if let Err(e) = fs::create_dir_all(parent) {
			log::error!("Failed to create output directory: {}", e);
			return;
		}
	}

	if let Err(e) = fs::create_dir_all(payload_dir) {
		log::error!("Failed to create Payload directory: {}", e);
	}
}

fn copy_resources(extension_path: &PathBuf, payload_dir: &Path) {
	let res_dir = extension_path.join("res");
	if res_dir.exists() {
		for entry in fs::read_dir(&res_dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_file() {
				let file_name = path.file_name().unwrap();
				let dest_path = payload_dir.join(file_name);
				if let Err(e) = fs::copy(&path, &dest_path) {
					log::warn!("Failed to copy resource {:?}: {}", path, e);
				}
			}
		}
	} else {
		log::warn!("res/ directory does not exist for {:?}", extension_path);
	}
}

fn copy_wasm(payload_dir: &Path, target_dir: &Path) {
	let wasm_files: Vec<_> = fs::read_dir(target_dir)
		.unwrap()
		.filter_map(|entry| {
			let path = entry.unwrap().path();
			if path.extension().map(|ext| ext == "wasm").unwrap_or(false) {
				Some(path)
			} else {
				None
			}
		})
		.collect();

	if wasm_files.is_empty() {
		log::error!("No wasm file found after build.");
		return;
	}

	let wasm_src = &wasm_files[0];
	let wasm_dst = payload_dir.join("main.wasm");

	if let Err(e) = fs::copy(wasm_src, &wasm_dst) {
		log::error!("Failed to copy wasm file: {}", e);
	}
}

fn copy_icon(extension_path: &Path, project_root: &Path, metadata_slug: &str, id: &str) {
	let icon_src = extension_path.join("res").join("icon.png");
	let icon_dst = project_root.join("extensions").join(id).join("icon.png");

	if !icon_src.exists() {
		log::warn!("icon.png not found for slug {}", metadata_slug);
		return;
	}

	match fs::copy(&icon_src, &icon_dst) {
		Ok(_) => {}
		Err(e) => log::error!("Failed to copy icon for {}: {}", metadata_slug, e),
	}
}
