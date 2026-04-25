use adw::prelude::*;
use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::{gio, glib};

use super::window::LoadoutWindow;
use crate::config::VERSION;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct LoadoutApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for LoadoutApplication {
        const NAME: &'static str = "LoadoutApplication";
        type Type = super::LoadoutApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for LoadoutApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<control>q"]);
        }
    }

    impl ApplicationImpl for LoadoutApplication {
        fn activate(&self) {
            let application = self.obj();
            let window = application.active_window().unwrap_or_else(|| {
                let window = LoadoutWindow::new(&*application);
                window.upcast()
            });

            window.present();
        }
    }

    impl GtkApplicationImpl for LoadoutApplication {}
    impl AdwApplicationImpl for LoadoutApplication {}
}

glib::wrapper! {
    pub struct LoadoutApplication(ObjectSubclass<imp::LoadoutApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl LoadoutApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .property("resource-base-path", "/dev/drostina/Loadout")
            .build()
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        self.add_action_entries([quit_action, about_action]);
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutDialog::builder()
            .application_name("Loadout")
            .application_icon("dev.drostina.Loadout")
            .developer_name("Drostina")
            .version(VERSION)
            .developers(vec!["Drostina"])
            .translator_credits(&gettext("translator-credits"))
            .copyright("© 2026 Drostina")
            .build();

        about.present(Some(&window));
    }
}

