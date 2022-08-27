use std::error::Error;
use std::fs::{copy, create_dir_all};
use std::path::{Component, Path, PathBuf};

use rand::Rng;

// stolen from https://github.com/rust-lang/rfcs/issues/2208#issuecomment-342679694
pub fn normalize(p: &Path) -> PathBuf {
	let mut stack: Vec<Component> = Vec::new();
	for component in p.components() {
		match component {
			Component::CurDir => {}
			Component::ParentDir => {
				let top = stack.last().cloned();
				match top {
					Some(c) => {
						match c {
							Component::Prefix(_) => { stack.push(component); }
							Component::RootDir => {}
							Component::CurDir => { unreachable!(); }
							Component::ParentDir => { stack.push(component); }
							Component::Normal(_) => { let _ = stack.pop(); }
						}
					}
					None => { stack.push(component); }
				}
			}
			_ => { stack.push(component); }
		}
	}
	if stack.is_empty() { return PathBuf::from(Component::CurDir.as_os_str()); }
	let mut norm_path = PathBuf::new();
	for item in &stack { norm_path.push(item.as_os_str()); }
	norm_path
}

pub fn rand_str(size: usize) -> String {
	let mut rng = rand::thread_rng();
	(0..size).map(|_| rng.gen_range('a'..'z')).collect()
}

pub fn find_available_name(dir: impl AsRef<Path>) -> PathBuf {
	let dir = dir.as_ref();
	for len in 1..32usize {
		for _ in 0..10 {
			let path = dir.join(rand_str(len));
			if !path.exists() {
				return path;
			}
		}
	}
	panic!("Did you flooded shm folder?")
}

pub fn copy_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
	let src = src.as_ref();
	let dst = dst.as_ref();
	for ent in (src.read_dir()?).flatten() {
		if ent.metadata()?.is_dir() {
			copy_all(src.join(ent.file_name()), dst.join(ent.file_name()))?;
		} else {
			if let Ok(true) = dst.try_exists() {} else { create_dir_all(dst)?; }
			copy(src.join(ent.file_name()), dst.join(ent.file_name()))?;
		}
	}
	Ok(())
}