use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::icon;
use super::library::SteamGame;

pub fn read_game(path: PathBuf, steam_root: &Path, opts: &HashMap<String, String>, proton: &HashMap<String, String>) -> Option<SteamGame> {
    let contents = fs::read_to_string(path).ok()?;
    let name = quoted_values(&contents, "name").into_iter().next()?;

    if is_non_game(&name) {
        return None;
    }

    let appid = quoted_values(&contents, "appid").into_iter().next()?;

    Some(SteamGame {
        icon_path: icon::find(steam_root, &appid),
        launch_options: opts.get(&appid).cloned(),
        proton: proton.get(&appid).cloned(),
        steam_root: steam_root.to_path_buf(),
        appid,
        name,
    })
}

fn is_non_game(name: &str) -> bool {
    name.starts_with("Proton")
        || name.starts_with("Steam Linux Runtime")
        || name == "Steamworks Common Redistributables"
}

pub fn is_app_manifest(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.starts_with("appmanifest_") && name.ends_with(".acf"))
}

pub(super) fn quoted_values(contents: &str, key: &str) -> Vec<String> {
    contents
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('"').skip(1);
            let current_key = parts.next()?;
            let value = parts.nth(1)?;

            (current_key == key).then(|| value.to_string())
        })
        .collect()
}

