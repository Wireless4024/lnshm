use std::collections::HashMap;
use std::env::temp_dir;
use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions, read_link, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::exit;

use serde::{Deserialize, Serialize};
use symlink::{remove_symlink_dir, symlink_dir};

use util::copy_all;

use crate::cli::parse_args;
use crate::util::{find_available_name, normalize};

mod util;
mod cli;

fn main() -> Result<(), Box<dyn Error>> {
	let args = parse_args();

	if !PathBuf::from("config.toml").exists() {
		#[cfg(not(target_os = "windows"))]
			let shm_path = if PathBuf::from("/dev/shm").exists() {// check if /dev/shm existed otherwise use temp directory
			"/dev/shm/ln-shm".to_string()
		} else {
			temp_dir().join("ln-shm").to_string_lossy().to_string()
		};
		#[cfg(target_os = "windows")]
			let shm_path = {
			// Drive R: usually mounted as ramdisk drive
			if PathBuf::from("R:\\").exists() {
				"R:\\ln-shm".to_string()
			} else {
				temp_dir().join("ln-shm").to_string_lossy().to_string()
			}
		};
		println!("Can't find config file creating new file..");
		let data = Config {
			shm_path,
			configs: Default::default(),
		};
		let default_config = toml::to_string(&data)?;
		let mut file = File::create("config.toml")?;
		file.write_all(default_config.as_bytes())?;
		exit(0);
	}
	let mut file = OpenOptions::new().read(true).write(true).open("config.example.toml")?;
	let mut content = String::with_capacity(file.metadata()?.len() as _);
	file.read_to_string(&mut content)?;
	let mut data: Config = toml::from_str(&content).expect("Valid toml syntax in config file");
	data.apply()?;
	let result = toml::to_string(&data)?;
	if result != content {
		println!("Saving new configuration");
		file.seek(SeekFrom::Start(0))?;
		file.set_len(0)?;
		file.write_all(result.as_bytes())?;
	}
	Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
	/// ramdisk path
	shm_path: String,
	/// Pair of target:config
	#[serde(flatten)]
	configs: HashMap<String, LinkDirectory>,
}

impl Config {
	fn link(cfg: &mut LinkDirectory, path: &Path, shm_path: &str) -> Result<bool, Box<dyn Error>> {
		let mut changed = false;
		if let Some(source) = &cfg.source {
			symlink_dir(source, path)?
		} else {
			let source = if let Some(file_name) = path.file_name().map(|it| it.to_string_lossy().to_string()) {
				format!("{}/{}", shm_path, file_name)
			} else {
				find_available_name(shm_path).to_string_lossy().to_string()
			};
			create_dir_all(&source)?;
			changed = true;
			symlink_dir(&source, path)?;
			cfg.source = Some(source);
		};
		Ok(changed)
	}
	pub fn apply(&mut self) -> Result<bool, Box<dyn Error>> {
		let shm_path = &self.shm_path;
		if !Path::new(shm_path).exists() {
			create_dir_all(shm_path)?;
		}
		let mut changed = false;
		let mut remap = Vec::new();
		for (raw_path, cfg) in &mut self.configs {
			let path = Path::new(raw_path);

			if path.exists() {
				// check if linked correctly
				if let Ok(meta) = read_link(path) {
					let absolute = meta.as_path();
					if let Some(expected) = &cfg.source {
						if absolute.to_string_lossy().as_ref() != expected.as_str() {
							remove_symlink_dir(path)?;
							changed |= Self::link(cfg, path, shm_path)?;
						}
					}
				} else {
					// was directory so backup and link
					rename(&path, format!("{}.old", path.to_string_lossy()))?;
					changed |= Self::link(cfg, &path, shm_path)?;
				}
			} else {
				changed |= Self::link(cfg, path, shm_path)?;
			}
			let resolve_data_path = if let Some(data) = &cfg.data {
				if let Some(src) = &cfg.source {
					copy_all(data, src)?;
					!Path::new(data).is_absolute()
				} else {
					false
				}
			} else {
				false
			};
			if resolve_data_path {
				let old_data = cfg.data.take();
				cfg.data = match old_data {
					None => None,
					Some(data) => {
						Some(Path::new(&data).canonicalize()?.to_string_lossy().to_string())
					}
				}
			}
			if !path.is_absolute() {
				remap.push(raw_path.clone());
			}
		}
		for old in remap {
			if let Some(cfg) = self.configs.remove(&old) {
				let mut old_path = Path::new(".").canonicalize()?;
				old_path.push(old);
				self.configs.insert(normalize(&old_path).to_string_lossy().to_string(), cfg);
			};
		}
		Ok(changed)
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct LinkDirectory {
	/// Copy data from this folder if present
	data: Option<String>,
	/// Actual folder to symlink
	#[serde(skip_serializing_if = "Option::is_none")]
	source: Option<String>,
}