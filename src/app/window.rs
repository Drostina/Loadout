use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

mod imp {
    use super::*;

    #[derive(Debug, Default, gtk::CompositeTemplate)]
    #[template(resource = "/dev/drostina/Loadout/window.ui")]
    pub struct LoadoutWindow {
        #[template_child]
        pub games_list: TemplateChild<gtk::ListBox>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LoadoutWindow {
        const NAME: &'static str = "LoadoutWindow";
        type Type = super::LoadoutWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for LoadoutWindow {
        fn constructed(&self) {
            self.parent_constructed();
            self.obj().load_games();
            let win = self.obj().clone();
            glib::idle_add_local_once(move || {
                gtk::prelude::GtkWindowExt::set_focus(&win, None::<&gtk::Widget>);
            });
        }
    }
    impl WidgetImpl for LoadoutWindow {}
    impl WindowImpl for LoadoutWindow {}
    impl ApplicationWindowImpl for LoadoutWindow {}
    impl AdwApplicationWindowImpl for LoadoutWindow {}
}

glib::wrapper! {
    pub struct LoadoutWindow(ObjectSubclass<imp::LoadoutWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl LoadoutWindow {
    pub fn new<P: IsA<gtk::Application>>(application: &P) -> Self {
        glib::Object::builder()
            .property("application", application)
            .build()
    }

    fn load_games(&self) {
        let games = crate::steam::installed_games();
        let list = self.imp().games_list.get();

        let list_weak = list.downgrade();
        let click = gtk::GestureClick::new();
        click.connect_pressed(move |_, _, x, y| {
            if let Some(list) = list_weak.upgrade() {
                if list
                    .pick(x, y, gtk::PickFlags::DEFAULT)
                    .and_then(|w| w.ancestor(adw::EntryRow::static_type()))
                    .is_none()
                {
                    if let Some(root) = list.root().and_downcast::<gtk::Window>() {
                        gtk::prelude::GtkWindowExt::set_focus(&root, None::<&gtk::Widget>);
                    }
                }
            }
        });
        list.add_controller(click);

        if games.is_empty() {
            let row = adw::ActionRow::builder()
                .title("No installed Steam games found")
                .build();
            list.append(&row);
            return;
        }

        for game in games {
            list.append(&game_row(game));
        }
    }
}

fn game_row(game: crate::steam::SteamGame) -> adw::EntryRow {
    let row = adw::EntryRow::builder().title(&game.name).build();
    row.set_focusable(false);
    row.set_activatable(false);
    row.set_selectable(false);
    row.set_text(game.launch_options.as_deref().unwrap_or(""));
    row.set_show_apply_button(true);

    let proton_label = gtk::Label::new(Some(game.proton.as_deref().unwrap_or("—")));
    proton_label.add_css_class("dim-label");
    proton_label.add_css_class("caption");
    row.add_suffix(&proton_label);

    if let Some(path) = &game.icon_path {
        let image = gtk::Image::from_file(path);
        image.set_pixel_size(32);
        row.add_prefix(&image);
    }

    let appid = game.appid;
    let root = game.steam_root;

    let appid_apply = appid.clone();
    let root_apply = root.clone();
    row.connect_apply(move |r| {
        persist_row(r, &root_apply, &appid_apply);
    });

    let appid_activate = appid.clone();
    let root_activate = root.clone();
    row.connect_entry_activated(move |r| {
        persist_row(r, &root_activate, &appid_activate);
    });

    let appid_leave = appid;
    let root_leave = root;
    let fc = gtk::EventControllerFocus::new();
    let row_weak = row.downgrade();
    fc.connect_leave(move |_| {
        if let Some(r) = row_weak.upgrade() {
            persist_row(&r, &root_leave, &appid_leave);
        }
    });
    row.add_controller(fc);

    row
}

fn persist_row(row: &adw::EntryRow, steam_root: &std::path::Path, appid: &str) {
    crate::steam::update_launch_options(steam_root, appid, &row.text());
    let pos = row.position();
    row.select_region(pos, pos);
}

