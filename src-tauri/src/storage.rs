use std::fs;
use std::path::PathBuf;
use dirs::config_dir;

pub fn get_storage_path() -> PathBuf {
    let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("zero-day-chat");
    let _ = fs::create_dir_all(&path);
    path.push("identity.json");
    path
}

pub fn save_local_data(data: &str) -> Result<(), std::io::Error> {
    fs::write(get_storage_path(), data)
}
