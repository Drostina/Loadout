use std::collections::HashMap;
use std::fs;
use std::path::Path;

use super::localconfig::{indent, is_section, key, quoted_pair};

pub struct ProtonTool {
    pub display: String,
    pub id: String,
}

pub fn available(steam_root: &Path) -> Vec<ProtonTool> {
    let mut tools: Vec<ProtonTool> = Vec::new();

    let common = steam_root.join("steamapps/common");
    if let Ok(entries) = fs::read_dir(&common) {
        for entry in entries.filter_map(Result::ok) {
            let manifest = entry.path().join("toolmanifest.vdf");
            if let Ok(contents) = fs::read_to_string(manifest) {
                if contents.lines().any(|l| l.contains("commandline") && l.contains("/proton")) {
                    if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                        tools.push(ProtonTool { display: name.to_string(), id: normalize(name) });
                    }
                }
            }
        }
    }

    let custom = steam_root.join("compatibilitytools.d");
    if let Ok(entries) = fs::read_dir(custom) {
        for entry in entries.filter_map(Result::ok) {
            let vdf = entry.path().join("compatibilitytool.vdf");
            let Ok(contents) = fs::read_to_string(vdf) else { continue };
            let mut depth = 0i32;
            let mut id = String::new();
            let mut display = String::new();
            let mut from_windows = false;
            for line in contents.lines() {
                let t = line.trim();
                if t == "{" { depth += 1; continue; }
                if t == "}" { depth -= 1; continue; }
                if let Some((k, v)) = quoted_pair(t) {
                    if depth == 2 && v.is_empty() { id = k.to_string(); }
                    if k == "display_name" && !v.is_empty() { display = v.to_string(); }
                    if k == "from_oslist" && v == "windows" { from_windows = true; }
                }
            }
            if from_windows && !id.is_empty() {
                tools.push(ProtonTool { display: if display.is_empty() { id.clone() } else { display }, id });
            }
        }
    }

    tools.sort_by(|a, b| a.display.cmp(&b.display));
    tools.dedup_by(|a, b| a.id == b.id);
    tools
}

fn normalize(name: &str) -> String {
    name.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("_")
}

pub fn versions(steam_root: &Path) -> HashMap<String, String> {
    let Ok(contents) = fs::read_to_string(steam_root.join("config/config.vdf")) else {
        return HashMap::new();
    };
    let mut map = HashMap::new();
    let mut in_compat = false;
    let mut compat_depth: Option<i32> = None;
    let mut depth = 0i32;
    let mut current_appid: Option<String> = None;
    for line in contents.lines() {
        let trimmed = line.trim();
        if !in_compat && is_section(trimmed, "CompatToolMapping") {
            in_compat = true;
        } else if in_compat && compat_depth.is_none() && trimmed == "{" {
            compat_depth = Some(depth + 1);
        } else if compat_depth == Some(depth) {
            if trimmed == "}" {
                compat_depth = None;
                in_compat = false;
            } else if let Some((k, v)) = quoted_pair(trimmed) {
                if v.is_empty() && k.chars().all(|c| c.is_ascii_digit()) {
                    current_appid = Some(k.to_string());
                }
            }
        } else if compat_depth.is_some() {
            if let Some((k, v)) = quoted_pair(trimmed) {
                if k == "name" && !v.is_empty() {
                    if let Some(appid) = &current_appid {
                        map.entry(appid.clone()).or_insert_with(|| v.to_string());
                    }
                }
            }
        }
        if trimmed == "{" { depth += 1; } else if trimmed == "}" { depth -= 1; }
    }
    map
}

pub fn update(steam_root: &Path, appid: &str, tool: &str) {
    let path = steam_root.join("config/config.vdf");
    if let Ok(contents) = fs::read_to_string(&path) {
        let updated = rewrite_compat_tool(&contents, appid, tool);
        if updated != contents {
            let _ = fs::write(&path, updated);
        }
    }
}

fn rewrite_compat_tool(contents: &str, appid: &str, tool: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut found_compat = false;
    let mut compat_depth: Option<i32> = None;
    let mut found_app = false;
    let mut app_depth: Option<i32> = None;
    let mut name_seen = false;
    let mut app_added = false;

    for line in contents.lines() {
        let trimmed = line.trim();

        if found_app && trimmed == "{" {
            app_depth = Some(depth + 1);
            found_app = false;
        } else if app_depth == Some(depth) && key(trimmed) == Some("name") {
            if !tool.is_empty() {
                lines.push(format!("{}\"name\"\t\t\"{}\"", indent(line), tool));
            }
            name_seen = true;
            continue;
        } else if app_depth == Some(depth) && trimmed == "}" {
            if !name_seen && !tool.is_empty() {
                lines.push(format!("{}\"name\"\t\t\"{}\"", indent(line), tool));
            }
            app_depth = None;
            name_seen = false;
            app_added = true;
        } else if found_compat && trimmed == "{" {
            compat_depth = Some(depth + 1);
            found_compat = false;
        } else if compat_depth == Some(depth) && is_section(trimmed, appid) {
            found_app = true;
        } else if compat_depth == Some(depth) && trimmed == "}" {
            if !app_added && !tool.is_empty() {
                let ind = indent(line);
                lines.push(format!("{}\t\"{}\"", ind, appid));
                lines.push(format!("{}\t{{", ind));
                lines.push(format!("{}\t\t\"name\"\t\t\"{}\"", ind, tool));
                lines.push(format!("{}\t\t\"config\"\t\t\"\"", ind));
                lines.push(format!("{}\t\t\"priority\"\t\t\"250\"", ind));
                lines.push(format!("{}\t}}", ind));
            }
            compat_depth = None;
        } else if is_section(trimmed, "CompatToolMapping") {
            found_compat = true;
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
