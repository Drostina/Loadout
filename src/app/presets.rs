use gtk::gio;
use gtk::prelude::SettingsExtManual;

const PRESETS_KEY: &str = "launch-presets";
const PRESET_SEPARATOR: &str = "\t";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LaunchPreset {
    pub name: String,
    pub command: String,
}

impl LaunchPreset {
    pub fn new(name: String, command: String) -> Self {
        Self { name, command }
    }
}

pub fn load(settings: &gio::Settings) -> Vec<LaunchPreset> {
    let mut presets = settings
        .strv(PRESETS_KEY)
        .iter()
        .filter_map(|raw| decode(raw.as_ref()))
        .collect::<Vec<_>>();
    presets.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    presets
}

pub fn save(settings: &gio::Settings, presets: &[LaunchPreset]) -> bool {
    let encoded = presets.iter().map(encode).collect::<Vec<_>>();
    settings.set_strv(PRESETS_KEY, encoded).is_ok()
}

fn encode(preset: &LaunchPreset) -> String {
    let safe_name = preset.name.replace(PRESET_SEPARATOR, " ");
    let safe_command = preset.command.replace('\n', " ");
    format!("{safe_name}{PRESET_SEPARATOR}{safe_command}")
}

fn decode(raw: &str) -> Option<LaunchPreset> {
    let (name, command) = raw.split_once(PRESET_SEPARATOR)?;
    let name = name.trim();
    let command = command.trim();
    if name.is_empty() || command.is_empty() {
        return None;
    }
    Some(LaunchPreset::new(name.to_string(), command.to_string()))
}
