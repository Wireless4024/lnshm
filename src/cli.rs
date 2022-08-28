use std::env::{current_dir, temp_dir};
use std::error::Error;
use std::fs::{create_dir_all, File, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{PathBuf};

use clap::{Command, CommandFactory, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};

use crate::{Config, normalize};

#[derive(Parser, Debug)]
#[clap(version, about, long_about = None)]
pub(crate) struct Args {
	/// Generate completion script
	#[clap(long = "generate", value_enum)]
	generator: Option<Shell>,
	/// Run as system mode (eg. systemd hook on linux)
	#[clap(long)]
	system: bool,

	/// Path to config file
	#[clap(short, long, value_hint = ValueHint::DirPath)]
	config: Option<String>,

	/// Print information and exit
	#[clap(short, long)]
	pub info: bool,

	/// Path to source directory (copy content into ramdisk on mount)
	#[clap(short, long, value_hint = ValueHint::DirPath)]
	pub(crate) source: Option<String>,

	/// Unlink / remove instead of create (ignore source option)
	#[clap(short, long)]
	pub(crate) remove: bool,

	/// target folder to link to ramdisk
	#[clap(value_parser, value_hint = ValueHint::DirPath)]
	pub(crate) link_target: Option<String>,
}

pub(crate) fn parse_args() -> Args {
	let args: Args = Args::parse();
	if let Some(generator) = args.generator {
		let mut cmd = Args::command();
		if generator == Shell::Bash {
			eprintln!("Saving completion file to ~.local/share/bash-completion/completions");
		} else {
			eprintln!("Generating completion file for {:?}...", generator);
		}
		print_completions(generator, &mut cmd, if generator == Shell::Bash { Some(dirs::home_dir().expect("Home dir").join(".local/share/bash-completion/completions/lnshm")) } else { None });
	}
	args
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command, save: Option<PathBuf>) {
	if let Some(save) = save {
		let mut file = if let Ok(true) = save.try_exists() {
			OpenOptions::new().write(true).truncate(true).open(save).expect("Open completion file")
		} else {
			create_dir_all(save.parent().expect("parent dir")).expect("Create completion folder");
			File::create(save).expect("Open completion file")
		};
		generate(gen, cmd, cmd.get_name().to_string(), &mut file);
	} else {
		generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
	}
}

impl Args {
	pub fn get_config(&self) -> Result<InternalConfig, Box<dyn Error>> {
		let config_file = if let Some(config) = &self.config {
			if config.starts_with('/') {
				PathBuf::from(config)
			} else {
				normalize(&current_dir()?.join(config))
			}
		} else if self.system && cfg!(target_os = "linux") {
			PathBuf::from("/etc/lnshm/config.toml")
		} else {
			dirs::home_dir().map(|it| it.join(".config/lnshm/config.toml")).unwrap_or_else(|| PathBuf::from("config.toml"))
		};

		let config_path = normalize(&config_file);

		if !config_path.exists() {
			if let Some(parent) = config_path.parent() {
				create_dir_all(parent)?;
			}
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