use std::fs;
use std::path::{Path, PathBuf};

pub fn find(steam_root: &Path, appid: &str) -> Option<PathBuf> {
    let dir = steam_root.join(format!("appcache/librarycache/{appid}"));
    fs::read_dir(dir).ok()?.filter_map(Result::ok).find_map(|entry| {
        let path = entry.path();
        let name = path.file_name()?.to_str()?;
        (name.len() == 44 && name.ends_with(".jpg")).then_some(path)
    })
}

