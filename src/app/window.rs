use std::cell::{OnceCell, RefCell};

use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use super::presets::{self, LaunchPreset};

mod imp {
    use super::*;

    #[derive(Debug, gtk::CompositeTemplate)]
    #[template(resource = "/dev/drostina/Loadout/window.ui")]
    pub struct LoadoutWindow {
        #[template_child]
        pub split_view: TemplateChild<adw::OverlaySplitView>,
        #[template_child]
        pub content_stack: TemplateChild<gtk::Stack>,
        #[template_child]
        pub page_title_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub sidebar_toggle: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub sidebar_list: TemplateChild<gtk::ListBox>,
        pub games_list: OnceCell<gtk::ListBox>,
        pub presets_list: OnceCell<gtk::ListBox>,
        pub preset_name_entry: OnceCell<adw::EntryRow>,
        pub preset_command_entry: OnceCell<adw::EntryRow>,
        pub add_preset_button: OnceCell<gtk::Button>,
        pub settings: gio::Settings,
        pub presets: RefCell<Vec<LaunchPreset>>,
    }

    impl Default for LoadoutWindow {
        fn default() -> Self {
            Self {
                split_view: Default::default(),
                content_stack: Default::default(),
                page_title_label: Default::default(),
                sidebar_toggle: Default::default(),
                sidebar_list: Default::default(),
                games_list: OnceCell::new(),
                presets_list: OnceCell::new(),
                preset_name_entry: OnceCell::new(),
                preset_command_entry: OnceCell::new(),
                add_preset_button: OnceCell::new(),
                settings: gio::Settings::new("dev.drostina.Loadout"),
                presets: RefCell::new(Vec::new()),
            }
        }
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
            self.obj().setup_stack_pages();
            self.obj().setup_sidebar();
            self.obj().setup_sidebar_toggle();
            self.obj().load_presets();
            self.obj().load_games();
            self.obj().setup_presets_ui();
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
        let list = self.imp().games_list.get().expect("games_list not initialized");
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }
        let presets = self.imp().presets.borrow().clone();
        super::games::populate(list, &presets);
    }

    fn setup_stack_pages(&self) {
        let stack = self.imp().content_stack.get();
        let games_builder = gtk::Builder::from_resource("/dev/drostina/Loadout/games-page.ui");
        let games_page: gtk::ScrolledWindow = games_builder.object("games_page").unwrap();
        let games_list: gtk::ListBox = games_builder.object("games_list").unwrap();
        stack.add_titled(&games_page, Some("games"), "Games");
        let _ = self.imp().games_list.set(games_list);

        let presets_builder =
            gtk::Builder::from_resource("/dev/drostina/Loadout/presets-page.ui");
        let presets_page: gtk::ScrolledWindow = presets_builder.object("presets_page").unwrap();
        let presets_list: gtk::ListBox = presets_builder.object("presets_list").unwrap();
        let preset_name_entry: adw::EntryRow = presets_builder.object("preset_name_entry").unwrap();
        let preset_command_entry: adw::EntryRow =
            presets_builder.object("preset_command_entry").unwrap();
        let add_preset_button: gtk::Button = presets_builder.object("add_preset_button").unwrap();
        stack.add_titled(&presets_page, Some("presets"), "Presets");
        let _ = self.imp().presets_list.set(presets_list);
        let _ = self.imp().preset_name_entry.set(preset_name_entry);
        let _ = self.imp().preset_command_entry.set(preset_command_entry);
        let _ = self.imp().add_preset_button.set(add_preset_button);
        stack.set_visible_child_name("games");
    }

    fn setup_sidebar(&self) {
        let stack = self.imp().content_stack.get();
        let title = self.imp().page_title_label.get();
        let split = self.imp().split_view.get();
        self.imp().sidebar_list.connect_row_selected(move |_, row| {
            let Some(row) = row else {
                return;
            };
            match row.index() {
                0 => {
                    stack.set_visible_child_name("games");
                    title.set_label("Games");
                }
                1 => {
                    stack.set_visible_child_name("presets");
                    title.set_label("Presets");
                }
                _ => {}
            }
            if split.is_collapsed() {
                split.set_show_sidebar(false);
            }
        });
        if let Some(row) = self.imp().sidebar_list.row_at_index(0) {
            self.imp().sidebar_list.select_row(Some(&row));
        }
    }

    fn setup_sidebar_toggle(&self) {
        let split = self.imp().split_view.get();
        let toggle = self.imp().sidebar_toggle.get();
        split.bind_property("show-sidebar", &toggle, "active").build();
        toggle.connect_toggled(move |btn| {
            split.set_show_sidebar(btn.is_active());
        });
    }

    fn load_presets(&self) {
        let presets = presets::load(&self.imp().settings);
        self.imp().presets.replace(presets);
        self.render_presets_list();
    }

    fn save_presets(&self) {
        let presets = self.imp().presets.borrow().clone();
        let _ = presets::save(&self.imp().settings, &presets);
        self.render_presets_list();
        self.load_games();
    }

    fn setup_presets_ui(&self) {
        let add_button = self
            .imp()
            .add_preset_button
            .get()
            .expect("add_preset_button not initialized");
        let win = self.clone();
        add_button.connect_clicked(move |_| {
            win.add_preset_from_inputs();
        });
    }

    fn add_preset_from_inputs(&self) {
        let name_entry = self
            .imp()
            .preset_name_entry
            .get()
            .expect("preset_name_entry not initialized");
        let command_entry = self
            .imp()
            .preset_command_entry
            .get()
            .expect("preset_command_entry not initialized");
        let name = name_entry.text().trim().to_string();
        let command = command_entry.text().trim().to_string();
        if name.is_empty() || command.is_empty() {
            return;
        }

        {
            let mut presets = self.imp().presets.borrow_mut();
            if let Some(existing) = presets.iter_mut().find(|p| p.name == name) {
                existing.command = command;
            } else {
                presets.push(LaunchPreset::new(name, command));
            }
        }

        name_entry.set_text("");
        command_entry.set_text("");
        self.save_presets();
    }

    fn render_presets_list(&self) {
        let list = self
            .imp()
            .presets_list
            .get()
            .expect("presets_list not initialized");
        while let Some(child) = list.first_child() {
            list.remove(&child);
        }

        let presets = self.imp().presets.borrow().clone();
        if presets.is_empty() {
            list.append(
                &adw::ActionRow::builder()
                    .title("No presets yet")
                    .subtitle("Create one above to reuse launch options quickly.")
                    .build(),
            );
            return;
        }

        for preset in presets {
            let row = adw::ActionRow::builder()
                .title(&preset.name)
                .subtitle(&preset.command)
                .build();

            let delete_button = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .tooltip_text("Delete preset")
                .valign(gtk::Align::Center)
                .build();

            let win = self.clone();
            let name = preset.name.clone();
            delete_button.connect_clicked(move |_| {
                win.remove_preset_by_name(&name);
            });

            row.add_suffix(&delete_button);
            list.append(&row);
        }
    }

    fn remove_preset_by_name(&self, name: &str) {
        self.imp().presets.borrow_mut().retain(|p| p.name != name);
        self.save_presets();
    }
}
