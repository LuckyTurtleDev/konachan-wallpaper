use anyhow::{self, Context};
use std::{fs, path::Path};

pub fn read_to_string<P: AsRef<Path>>(path: P) -> anyhow::Result<String> {
	let path = path.as_ref();
	fs::read_to_string(path).with_context(|| format!("Failed to open {}", path.to_string_lossy()))
}
