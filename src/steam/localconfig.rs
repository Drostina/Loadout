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
    let mut depth = 0;
    let mut app_depth = None;
    let mut found_app = false;
    let mut launch_options_seen = false;

    for line in contents.lines() {
        let trimmed = line.trim();

        if found_app && trimmed == "{" {
            app_depth = Some(depth + 1);
            found_app = false;
        } else if app_depth == Some(depth) && key(trimmed) == Some("LaunchOptions") {
            push_launch_options(&mut lines, line, value);
            launch_options_seen = true;
            continue;
        } else if app_depth == Some(depth) && trimmed == "}" {
            if !launch_options_seen {
                push_launch_options(&mut lines, line, value);
            }
            app_depth = None;
            launch_options_seen = false;
        } else if is_section(trimmed, appid) {
            found_app = true;
        }

        if trimmed == "{" {
            depth += 1;
        } else if trimmed == "}" {
            depth -= 1;
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

pub(super) fn is_section(line: &str, expected: &str) -> bool {
    let Some((key, value)) = quoted_pair(line) else {
        return false;
    };
    key == expected && value.is_empty()
}

pub(super) fn key(line: &str) -> Option<&str> {
    quoted_pair(line).map(|(key, _)| key)
}

pub(super) fn quoted_pair(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split('"').skip(1);
    let key = parts.next()?;
    Some((key, parts.nth(1).unwrap_or("")))
}

pub(super) fn indent(line: &str) -> &str {
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

pub(super) fn parse(contents: &str, field: &str, map: &mut HashMap<String, String>) {
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
