use std::rc::Rc;

use adw::prelude::*;
use gtk::prelude::*;

use crate::steam::{self, SteamGame};

pub fn populate(list: &gtk::ListBox) {
    let list_w = list.downgrade();
    let click = gtk::GestureClick::new();
    click.connect_pressed(move |_, _, x, y| {
        let Some(list) = list_w.upgrade() else { return };
        let hit_entry = list
            .pick(x, y, gtk::PickFlags::DEFAULT)
            .and_then(|w| w.ancestor(adw::EntryRow::static_type()))
            .is_some();
        if !hit_entry {
            if let Some(win) = list.root().and_downcast::<gtk::Window>() {
                gtk::prelude::GtkWindowExt::set_focus(&win, None::<&gtk::Widget>);
            }
        }
    });
    list.add_controller(click);

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
    let row_h = gtk::SizeGroup::new(gtk::SizeGroupMode::Vertical);
    let proton_w = gtk::SizeGroup::new(gtk::SizeGroupMode::Horizontal);
    for g in games {
        list.append(&row(g, &tools, &row_h, &proton_w));
    }
}

fn proton_factory(tools: Rc<Vec<steam::ProtonTool>>) -> gtk::SignalListItemFactory {
    let f = gtk::SignalListItemFactory::new();
    f.connect_setup(|_, o| {
        o.downcast_ref::<gtk::ListItem>()
            .unwrap()
            .set_child(Some(&gtk::Label::builder().xalign(0.0).build()));
    });
    f.connect_bind(move |_, o| {
        let item = o.downcast_ref::<gtk::ListItem>().unwrap();
        let label = item.child().unwrap().downcast::<gtk::Label>().unwrap();
        let i = item.position() as usize;
        label.set_text(match i {
            0 => "Native",
            _ => tools.get(i - 1).map(|t| t.display.as_str()).unwrap_or(""),
        });
    });
    f
}

fn row(
    g: SteamGame,
    tools: &Rc<Vec<steam::ProtonTool>>,
    row_h: &gtk::SizeGroup,
    proton_w: &gtk::SizeGroup,
) -> adw::EntryRow {
    let SteamGame {
        name,
        appid,
        steam_root,
        icon_path,
        launch_options,
        proton,
    } = g;

    let row = adw::EntryRow::builder().title(&name).build();
    row.set_text(launch_options.as_deref().unwrap_or(""));

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
    dd.connect_selected_item_notify(move |d| {
        let i = d.selected() as usize;
        let id = if i == 0 {
            ""
        } else {
            tools_c.get(i - 1).map(|t| t.id.as_str()).unwrap_or("")
        };
        steam::update_proton_version(&root, &aid, id);
    });

    row.add_suffix(&dd);
    row_h.add_widget(&row);
    proton_w.add_widget(&dd);

    let root = steam_root;
    let aid = appid;
    let row_w = row.downgrade();
    let fc = gtk::EventControllerFocus::new();
    fc.connect_leave(move |_| {
        if let Some(r) = row_w.upgrade() {
            steam::update_launch_options(&root, &aid, &r.text());
        }
    });
    row.delegate().unwrap().add_controller(fc);
    row.connect_entry_activated(|r| {
        r.parent().map(|p| p.grab_focus());
    });

    row
}
