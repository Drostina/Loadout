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
            self.obj().connect_map(|win| {
                let win = win.clone();
                glib::idle_add_local_once(move || {
                    if let Some(w) = win.focus().filter(|w| w.is::<gtk::Text>()) {
                        w.downcast::<gtk::Text>().unwrap().select_region(0, 0);
                    }
                    gtk::prelude::GtkWindowExt::set_focus(&win, None::<&gtk::Widget>);
                });
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
        super::games::populate(&self.imp().games_list.get());
    }
}
