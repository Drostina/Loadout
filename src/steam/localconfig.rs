use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn update_launch_options(steam_root: &Path, appid: &str, value: &str) {
    let Ok(users) = fs::read_dir(steam_root.join("userdata")) else {
        return;
    };

    for entry in users.filter_map(Result::ok) {
        let path = entry.path().join("config/localconfig.vdf");
        if let Ok(contents) = fs::read_to_string(&path) {
            let updated = rewrite_launch_options(&contents, appid, value);
            if updated != contents {
                let _ = fs::write(&path, updated);
            }
        }
    }
}

fn rewrite_launch_options(contents: &str, appid: &str, value: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut waiting_for_app = false;
    let mut in_app = false;
    let mut app_depth = 0;
    let mut launch_options_seen = false;

    for line in contents.lines() {
        let trimmed = line.trim();

        if in_app && trimmed == "}" {
            if app_depth == 1 && !launch_options_seen {
                push_launch_options(&mut lines, line, value);
            }
            app_depth -= 1;
            if app_depth == 0 {
                in_app = false;
                launch_options_seen = false;
            }
        } else if in_app && app_depth == 1 && key(trimmed) == Some("LaunchOptions") {
            push_launch_options(&mut lines, line, value);
            launch_options_seen = true;
            continue;
        } else if in_app && trimmed == "{" {
            app_depth += 1;
        } else if waiting_for_app && trimmed == "{" {
            waiting_for_app = false;
            in_app = true;
            app_depth = 1;
        } else if is_section(trimmed, appid) {
            waiting_for_app = true;
        }

        lines.push(line.to_string());
    }

    let joined = lines.join("\n");
    if contents.ends_with('\n') { joined + "\n" } else { joined }
}

fn push_launch_options(lines: &mut Vec<String>, line: &str, value: &str) {
    if !value.is_empty() {
        let value = value.replace('\\', "\\\\").replace('"', "\\\"");
        lines.push(format!("{}\"LaunchOptions\"\t\t\"{}\"", indent(line), value));
    }
}

fn is_section(line: &str, expected: &str) -> bool {
    let Some((key, value)) = quoted_pair(line) else {
        return false;
    };
    key == expected && value.is_empty()
}

fn key(line: &str) -> Option<&str> {
    quoted_pair(line).map(|(key, _)| key)
}

fn quoted_pair(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split('"').skip(1);
    let key = parts.next()?;
    Some((key, parts.nth(1).unwrap_or("")))
}

fn indent(line: &str) -> &str {
    let trimmed = line.trim_start();
    &line[..line.len() - trimmed.len()]
}

pub fn launch_options(steam_root: &Path) -> HashMap<String, String> {
    let Ok(users) = fs::read_dir(steam_root.join("userdata")) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    for entry in users.filter_map(Result::ok) {
        let path = entry.path().join("config/localconfig.vdf");
        if let Ok(contents) = fs::read_to_string(path) {
            parse(&contents, "LaunchOptions", &mut map);
        }
    }
    map
}

pub fn proton_versions(steam_root: &Path) -> HashMap<String, String> {
    let Ok(contents) = fs::read_to_string(steam_root.join("config/config.vdf")) else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    parse(&contents, "name", &mut map);
    map
}

fn parse(contents: &str, field: &str, map: &mut HashMap<String, String>) {
    let mut current_appid: Option<String> = None;

    for line in contents.lines() {
        let Some((key, value)) = quoted_pair(line.trim()) else {
            continue;
        };

        if value.is_empty() && key.chars().all(|c| c.is_ascii_digit()) {
            current_appid = Some(key.to_string());
        } else if key == field && !value.is_empty() {
            if let Some(appid) = &current_appid {
                map.entry(appid.clone()).or_insert_with(|| value.to_string());
            }
        }
    }
}

