use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn launch_options(steam_root: &Path) -> HashMap<String, String> {
    let Ok(users) = fs::read_dir(steam_root.join("userdata")) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    for entry in users.filter_map(Result::ok) {
        let path = entry.path().join("config/localconfig.vdf");
        if let Ok(contents) = fs::read_to_string(path) {
            parse(&contents, &mut map);
        }
    }
    map
}

fn parse(contents: &str, map: &mut HashMap<String, String>) {
    let mut current_appid: Option<String> = None;

    for line in contents.lines() {
        let mut parts = line.trim().split('"').skip(1);
        let Some(key) = parts.next() else { continue };
        let value = parts.nth(1).unwrap_or("");

        if value.is_empty() && key.chars().all(|c| c.is_ascii_digit()) {
            current_appid = Some(key.to_string());
        } else if key == "LaunchOptions" && !value.is_empty() {
            if let Some(appid) = &current_appid {
                map.insert(appid.clone(), value.to_string());
            }
        }
    }
}

