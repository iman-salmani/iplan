use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{gdk, gio, glib};
use std::fs;

use crate::db::check_database;
use crate::IPlanApplication;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/backup_window.ui")]
    pub struct BackupWindow {
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for BackupWindow {
        const NAME: &'static str = "BackupWindow";
        type Type = super::BackupWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for BackupWindow {}
    impl WidgetImpl for BackupWindow {}
    impl WindowImpl for BackupWindow {}
    impl AdwWindowImpl for BackupWindow {}
}

glib::wrapper! {
    pub struct BackupWindow(ObjectSubclass<imp::BackupWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl BackupWindow {
    pub fn new(application: &IPlanApplication, app_window: &gtk::Window) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        win
    }

    #[template_callback]
    fn export_activated(&self, _: adw::ActionRow) {
        let dialog = gtk::FileDialog::new();
        dialog.set_accept_label(Some(&gettext("Export")));
        let now = glib::DateTime::now_local().unwrap();
        dialog.set_initial_name(Some(&now.format("IPlan data %F %R.db").unwrap()));
        let toast_overlay = self.imp().toast_overlay.to_owned();
        dialog.save(
            Some(self),
            Some(&gio::Cancellable::new()),
            glib::clone!(@weak toast_overlay => move |file| {
                if let Ok(file) = file {
                    let data_path = glib::user_data_dir().join("data.db");
                    let export_path = file.path().unwrap();
                    if let Err(err) = fs::copy(data_path, export_path) {
                        let toast = adw::Toast::new(&err.to_string());
                        toast_overlay.add_toast(toast);
                    }
                }
            }),
        );
    }

    #[template_callback]
    fn export_path_activated(&self, _: adw::ActionRow) {
        if let Some(display) = gdk::Display::default() {
            display
                .clipboard()
                .set_text(glib::user_data_dir().to_str().unwrap());

            self.imp()
                .toast_overlay
                .add_toast(adw::Toast::new(&gettext("Path copied!")));
        }
    }

    #[template_callback]
    fn import_activated(&self, _: adw::ActionRow) {
        let dialog = gtk::FileDialog::new();
        dialog.set_accept_label(Some(&gettext("Import")));
        dialog.open(
            Some(self),
            Some(&gio::Cancellable::new()),
            glib::clone!(@weak self as obj => move |file| {
                if let Ok(file) = file {
                    let now = glib::DateTime::now_local().unwrap();
                    let cache_filename = now.format("IPlan data %F %R.db").unwrap();
                    let cache_path = glib::user_cache_dir().join(cache_filename);
                    let data_path = glib::user_data_dir().join("data.db");
                    let import_path = file.path().unwrap();
                    let toast_overlay = obj.imp().toast_overlay.to_owned();
                    if let Err(err) = fs::copy(data_path.to_str().unwrap(), cache_path) {
                        let toast = adw::Toast::new(&format!("{}: {}", gettext("Error while caching previous data"), err));
                        toast_overlay.add_toast(toast);
                    } else if let Err(err) = fs::copy(import_path, data_path) {
                        let toast = adw::Toast::new(&format!("{}: {}", gettext("Error while importing data"), err));
                        toast_overlay.add_toast(toast);
                    } else {
                        check_database().expect("Database check has failed(after importing data)");
                        obj.transient_for().unwrap().activate_action("project.open", None).expect("Failed to send project.open action");
                    }
                }
            }),
        );
    }
}
