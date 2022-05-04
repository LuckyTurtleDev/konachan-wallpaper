use directories::{ProjectDirs, UserDirs};
use once_cell::sync::Lazy;
use std::path::PathBuf;

const CARGO_PKG_NAME: &'static str = env!("CARGO_PKG_NAME");
static PROJECT_DIRS: Lazy<ProjectDirs> =
	Lazy::new(|| ProjectDirs::from("de", "lukas1818", CARGO_PKG_NAME).expect("failed to get project dirs"));
pub static WALLPAPERS_FILE: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.data_dir().join("wallpapers.txt"));
pub static WALLPAPERS_FOLDER: Lazy<String> = Lazy::new(|| {
	match UserDirs::new() {
		Some(user_dir) => match user_dir.picture_dir() {
			Some(dir) => dir.join(format!("{}/", CARGO_PKG_NAME)).to_path_buf(),
			None => PROJECT_DIRS.data_dir().join("wallpapers/"),
		},
		None => PROJECT_DIRS.data_dir().join("wallpapers/"),
	}
	.to_string_lossy()
	.into_owned()
});
pub static CURRENT_WALLAPER_FILE: Lazy<PathBuf> = Lazy::new(|| PathBuf::from("/tmp/current-wallpaper.txt"));
pub static CONFIG_FILE: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.config_dir().join("config.txt"));
