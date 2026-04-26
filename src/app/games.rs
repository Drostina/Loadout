use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::glib;

use super::presets::LaunchPreset;
use crate::steam::{self, SteamGame};

fn subtitle_for_launch_options(text: &str) -> &str {
    if text.is_empty() {
        "No launch options set"
    } else {
        text
    }
}

pub fn populate(list: &gtk::ListBox, presets: &[LaunchPreset]) {
    let games = steam::installed_games();
    if games.is_empty() {
        list.append(
            &adw::ActionRow::builder()
                .title("No installed Steam games found")
                .build(),
        );
        return;
    }

    let tools = Rc::new(
        games
            .first()
            .map(|g| steam::available_proton_tools(&g.steam_root))
            .unwrap_or_default(),
    );
    let compatibility_w = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    let preset_w = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    list.append(&header_row(&compatibility_w, &preset_w));
    for g in games {
        list.append(&row(g, &tools, presets, &compatibility_w, &preset_w));
    }
}

fn header_row(
    compatibility_w: &gtk::SizeGroup,
    preset_w: &gtk::SizeGroup,
) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title("Game")
        .activatable(false)
        .selectable(false)
        .build();
    let box_ = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .valign(gtk::Align::Center)
        .build();
    let compatibility = gtk::Label::builder()
        .label("Compatibility")
        .xalign(0.0)
        .halign(gtk::Align::Start)
        .build();
    let preset = gtk::Label::builder()
        .label("Preset")
        .xalign(0.0)
        .halign(gtk::Align::Start)
        .build();
    compatibility_w.add_widget(&compatibility);
    preset_w.add_widget(&preset);
    box_.append(&compatibility);
    box_.append(&preset);
    row.add_suffix(&box_);
    row
}

fn proton_factory(tools: Rc<Vec<steam::ProtonTool>>) -> gtk::SignalListItemFactory {
    let f = gtk::SignalListItemFactory::new();
    f.connect_setup(|_, obj| {
        obj.downcast_ref::<gtk::ListItem>()
            .expect("proton factory setup expects gtk::ListItem")
            .set_child(Some(&gtk::Label::builder().xalign(0.0).build()));
    });
    f.connect_bind(move |_, obj| {
        let item = obj
            .downcast_ref::<gtk::ListItem>()
            .expect("proton factory bind expects gtk::ListItem");
        let label = item
            .child()
            .expect("proton factory list item child missing")
            .downcast::<gtk::Label>()
            .expect("proton factory child should be gtk::Label");
        let i = item.position() as usize;
        label.set_text(match i {
            0 => "Steam Default",
            _ => tools.get(i - 1).map(|t| t.display.as_str()).unwrap_or(""),
        });
    });
    f
}

fn row(
    g: SteamGame,
    tools: &Rc<Vec<steam::ProtonTool>>,
    presets: &[LaunchPreset],
    compatibility_w: &gtk::SizeGroup,
    preset_w: &gtk::SizeGroup,
) -> adw::ExpanderRow {
    let SteamGame {
        name,
        appid,
        steam_root,
        icon_path,
        launch_options,
        proton,
    } = g;

    let launch_text = launch_options.as_deref().unwrap_or("").to_string();
    let row = adw::ExpanderRow::builder()
        .title(&name)
        .subtitle(subtitle_for_launch_options(&launch_text))
        .build();

    if let Some(path) = icon_path {
        let img = gtk::Image::from_file(path);
        img.set_pixel_size(32);
        row.add_prefix(&img);
    }

    let ids: Vec<&str> = std::iter::once("")
        .chain(tools.iter().map(|t| t.id.as_str()))
        .collect();
    let model = gtk::StringList::new(&ids);
    let dd = gtk::DropDown::builder()
        .model(&model)
        .valign(gtk::Align::Center)
        .vexpand(false)
        .build();
    let fac = proton_factory(tools.clone());
    dd.set_factory(Some(&fac));
    dd.set_list_factory(Some(&fac));
    dd.set_selected(
        tools
            .iter()
            .position(|t| proton.as_ref() == Some(&t.id))
            .map(|i| (i + 1) as u32)
            .unwrap_or(0),
    );

    let tools_c = tools.clone();
    let root = steam_root.clone();
    let aid = appid.clone();
    dd.connect_selected_item_notify(move |dropdown| {
        let i = dropdown.selected() as usize;
        let id = if i == 0 {
            ""
        } else {
            tools_c.get(i - 1).map(|t| t.id.as_str()).unwrap_or("")
        };
        steam::update_proton_version(&root, &aid, id);
    });
    dd.set_tooltip_text(Some("Choose Proton compatibility tool"));

    let options_row = adw::EntryRow::builder().title("Launch options").build();
    options_row.set_text(&launch_text);
    row.add_row(&options_row);

    let preset_model_labels: Vec<&str> = std::iter::once("None")
        .chain(std::iter::once("Custom"))
        .chain(presets.iter().map(|p| p.name.as_str()))
        .collect();
    let preset_model = gtk::StringList::new(&preset_model_labels);
    let preset_dd = gtk::DropDown::builder()
        .model(&preset_model)
        .valign(gtk::Align::Center)
        .vexpand(false)
        .build();
    preset_dd.set_tooltip_text(Some("Apply saved launch options preset"));
    let preset_idx = if launch_text.is_empty() {
        0
    } else {
        presets
            .iter()
            .position(|p| p.command == launch_text)
            .map(|i| (i + 2) as u32)
            .unwrap_or(1)
    };
    preset_dd.set_selected(preset_idx);

    let applying_preset = Rc::new(Cell::new(false));
    let syncing_preset_selection = Rc::new(Cell::new(false));
    let pending_save = Rc::new(RefCell::new(None::<glib::SourceId>));
    let root = steam_root.clone();
    let aid = appid.clone();
    let row_w = row.downgrade();
    let options_w = options_row.downgrade();
    let preset_commands = Rc::new(
        presets
            .iter()
            .map(|p| p.command.clone())
            .collect::<Vec<_>>(),
    );
    let commands_c = preset_commands.clone();
    let applying_preset_c = applying_preset.clone();
    let syncing_preset_selection_c = syncing_preset_selection.clone();
    let pending_save_c = pending_save.clone();
    preset_dd.connect_selected_notify(move |dropdown| {
        if syncing_preset_selection_c.get() {
            return;
        }
        let i = dropdown.selected() as usize;
        if i == 1 {
            return;
        }
        let command = if i == 0 {
            ""
        } else {
            let Some(command) = commands_c.get(i - 2) else {
                return;
            };
            command.as_str()
        };
        if let (Some(r), Some(o)) = (row_w.upgrade(), options_w.upgrade()) {
            applying_preset_c.set(true);
            o.set_text(command);
            if !r.is_expanded() {
                r.set_subtitle(subtitle_for_launch_options(command));
            }
            applying_preset_c.set(false);
        }
        if let Some(source) = pending_save_c.borrow_mut().take() {
            source.remove();
        }
        steam::update_launch_options(&root, &aid, command);
    });
    let preset_dd_w = preset_dd.downgrade();
    let applying_preset_c = applying_preset.clone();
    let row_w = row.downgrade();
    let preset_commands_for_match = preset_commands.clone();
    let syncing_preset_selection_c = syncing_preset_selection.clone();
    let root_for_change = steam_root.clone();
    let aid_for_change = appid.clone();
    let pending_save_c = pending_save.clone();
    options_row.connect_changed(move |o| {
        if applying_preset_c.get() {
            return;
        }
        if let Some(dd) = preset_dd_w.upgrade() {
            let text = o.text();
            let idx = if text.is_empty() {
                0
            } else {
                preset_commands_for_match
                    .iter()
                    .position(|cmd| cmd == text.as_str())
                    .map(|i| (i + 2) as u32)
                    .unwrap_or(1)
            };
            syncing_preset_selection_c.set(true);
            dd.set_selected(idx);
            syncing_preset_selection_c.set(false);
        }
        if let Some(r) = row_w.upgrade() {
            let text = o.text();
            if !r.is_expanded() {
                r.set_subtitle(subtitle_for_launch_options(text.as_str()));
            }
        }
        if let Some(source) = pending_save_c.borrow_mut().take() {
            source.remove();
        }
        let text = o.text().to_string();
        let root = root_for_change.clone();
        let aid = aid_for_change.clone();
        let pending_save_done = pending_save_c.clone();
        let source = glib::timeout_add_local(Duration::from_millis(400), move || {
            steam::update_launch_options(&root, &aid, &text);
            pending_save_done.borrow_mut().take();
            glib::ControlFlow::Break
        });
        *pending_save_c.borrow_mut() = Some(source);
    });
    let options_w = options_row.downgrade();
    row.connect_expanded_notify(move |r| {
        if r.is_expanded() {
            r.set_subtitle("");
            return;
        }
        if let Some(o) = options_w.upgrade() {
            let text = o.text();
            r.set_subtitle(subtitle_for_launch_options(text.as_str()));
        }
    });
    let controls_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .valign(gtk::Align::Center)
        .build();
    compatibility_w.add_widget(&dd);
    preset_w.add_widget(&preset_dd);
    controls_box.append(&dd);
    controls_box.append(&preset_dd);
    row.add_suffix(&controls_box);

    row
}
