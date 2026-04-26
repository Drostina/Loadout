mod app;
mod config;
mod steam;

use app::LoadoutApplication;

use config::{GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::prelude::*;
use gtk::{gio, glib};

fn main() -> glib::ExitCode {
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/loadout.gresource")
        .expect("Could not load resources");
    gio::resources_register(&resources);

    let app = LoadoutApplication::new("io.github.Drostina.Loadout", &gio::ApplicationFlags::empty());

    app.run()
}
