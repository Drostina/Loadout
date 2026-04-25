use std::path::Path;

use adw::prelude::ActionRowExt;
use adw::subclass::prelude::*;
use gtk::prelude::*;
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
        let list = &self.imp().games_list;

        if games.is_empty() {
            list.append(&game_row("No installed Steam games found", None));
            return;
        }

        for game in games {
            list.append(&game_row(&game.name, game.icon_path.as_deref()));
        }
    }
}

fn game_row(name: &str, icon_path: Option<&Path>) -> adw::ActionRow {
    let row = adw::ActionRow::builder().title(name).build();

    if let Some(path) = icon_path {
        let image = gtk::Image::from_file(path);
        image.set_pixel_size(32);
        row.add_prefix(&image);
    }

    row
}

