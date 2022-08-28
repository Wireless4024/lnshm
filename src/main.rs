use std::collections::HashMap;
use std::error::Error;
use std::fs::{create_dir_all, OpenOptions, read_link, rename};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
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
	let cfg = args.get_config().expect("parse config");

	let mut file = OpenOptions::new().read(true).write(true).open(&cfg.config_file)?;
	let mut content = String::with_capacity(file.metadata()?.len() as _);
	file.read_to_string(&mut content)?;
	let mut data: Config = toml::from_str(&content).expect("Valid toml syntax in config file");
	if args.info {
		println!("Target <-> Source (ramdisk folder)");
		for (path, cfg) in data.configs {
			println!("{} <-> {}", path, cfg.source.unwrap_or_else(|| String::from("(nothing)")))
		}
		exit(0);
	}
	if let Some(target) = &args.link_target {
		if args.remove {
			data.unlink(target)?;
		} else {
			let ld = LinkDirectory {
				source: args.source,
				..Default::default()
			};
			data.add(target, ld);
		}
	}
	data.apply().expect("Apply change");
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
			if Path::new(source).exists() {
				symlink_dir(source, path)?;
			} else {
				cfg.source = None;
				return Self::link(cfg, path, shm_path);
			}
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

	pub fn add(&mut self, target: impl AsRef<str>, cfg: LinkDirectory) {
		self.configs.insert(target.as_ref().to_string(), cfg);
	}

	pub fn unlink(&mut self, target: impl AsRef<str>) -> Result<(), Box<dyn Error>> {
		let mut folder = Path::new(".").canonicalize()?;
		folder.push(target.as_ref());
		let folder = normalize(&folder);
		let folder_str = folder.to_string_lossy();
		let link = self.configs.remove(&*folder_str);
		// link existed
		if link.is_some() {
			// was a symlink
			if folder.exists() && folder.read_link().is_ok() {
				println!("Removed link {} <-> {}", folder_str, link.unwrap().source.unwrap_or_else(|| String::from("(nothing)")));
				remove_symlink_dir(folder)?;
			}
		} else {
			eprintln!("Target folder doesn't existed!");
			exit(1);
		}
		Ok(())
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LinkDirectory {
	/// Copy data from this folder if present
	data: Option<String>,
	/// Actual folder to symlink
	#[serde(skip_serializing_if = "Option::is_none")]
	source: Option<String>,
}