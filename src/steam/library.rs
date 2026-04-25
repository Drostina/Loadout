use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use super::manifest::{is_app_manifest, quoted_values, read_game};

#[derive(Debug, Clone)]
pub struct SteamGame {
    pub name: String,
    pub icon_path: Option<PathBuf>,
}

pub fn installed_games() -> Vec<SteamGame> {
    let mut games = steam_roots()
        .into_iter()
        .flat_map(|root| {
            let libs = library_paths(&root);
            libs.into_iter().flat_map(move |lib| read_games(lib, &root))
        })
        .collect::<Vec<_>>();

    games.sort_by_key(|game| game.name.to_lowercase());
    games.dedup_by(|a, b| a.name == b.name);
    games
}

fn steam_roots() -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };

    [
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ]
    .into_iter()
    .filter(|path| path.exists())
    .collect()
}

fn library_paths(root: &Path) -> Vec<PathBuf> {
    let mut libraries = vec![root.to_path_buf()];
    let library_file = root.join("steamapps/libraryfolders.vdf");
    let Ok(contents) = fs::read_to_string(library_file) else {
        return libraries;
    };

    libraries.extend(
        quoted_values(&contents, "path")
            .into_iter()
            .map(PathBuf::from)
            .filter(|path| path.exists()),
    );

    libraries
}

fn read_games(library: PathBuf, steam_root: &Path) -> Vec<SteamGame> {
    let steamapps = library.join("steamapps");
    let Ok(entries) = fs::read_dir(steamapps) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_app_manifest(path))
        .filter_map(|path| read_game(path, steam_root))
        .collect()
}

