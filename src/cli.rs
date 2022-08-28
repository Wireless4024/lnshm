use std::env::temp_dir;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;

use crate::Config;

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Args {
	/// Run as system mode (eg. systemd hook on linux)
	#[clap(long)]
	system: bool,

	/// Path to config file
	#[clap(short, long)]
	config: Option<String>,

	/// Path to source directory (copy content into ramdisk on mount)
	#[clap(short, long)]
	pub(crate) source: Option<String>,

	/// Unlink / remove instead of create (ignore source option)
	#[clap(short, long)]
	pub(crate) remove: bool,

	/// target folder to link to ramdisk
	#[clap(value_parser)]
	pub(crate) link_target: Option<String>,
}

pub(crate) fn parse_args() -> Args {
	Args::parse()
}

impl Args {
	pub fn get_config(&self) -> Result<InternalConfig, Box<dyn Error>> {
		let config_file = if let Some(config) = &self.config {
			PathBuf::from(config)
		} else if self.system && cfg!(target_os = "linux") {
			PathBuf::from("/etc/lnshm/config.toml")
		} else {
			dirs::home_dir().map(|it| it.join(".config/lnshm/config.toml")).unwrap_or_else(|| PathBuf::from("config.toml"))
		};

		if !PathBuf::from(&config_file).exists() {
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
			let mut file = File::create(&config_file)?;
			file.write_all(default_config.as_bytes())?;
		};

		Ok(InternalConfig {
			config_file
		})
	}
}

pub(crate) struct InternalConfig {
	pub config_file: PathBuf,
}