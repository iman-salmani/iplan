/* application.rs
 *
 * Copyright 2023 Iman Salmani
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use adw::subclass::prelude::*;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::config::{APPLICATION_ID, VERSION};
use crate::views::search::SearchWindow;
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct IPlanApplication {}

    #[glib::object_subclass]
    impl ObjectSubclass for IPlanApplication {
        const NAME: &'static str = "IPlanApplication";
        type Type = super::IPlanApplication;
        type ParentType = adw::Application;
    }

    impl ObjectImpl for IPlanApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.instance();
            obj.setup_gactions();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.shortcuts", &["<primary>question"]);
            obj.set_accels_for_action("app.search", &["<primary>f"]);
        }
    }

    impl ApplicationImpl for IPlanApplication {
        // We connect to the activate callback to create a window when the application
        // has been launched. Additionally, this callback notifies us when the user
        // tries to launch a "second instance" of the application. When they try
        // to do that, we'll just present any existing window.
        fn activate(&self) {
            let application = self.instance();
            // Get the current window or create one if necessary
            let window = if let Some(window) = application.active_window() {
                window
            } else {
                let window = IPlanWindow::new(&*application);
                if APPLICATION_ID == "ir.imansalmani.IPlan.Devel" {
                    window.add_css_class("devel")
                }
                window.upcast()
            };

            // Ask the window manager/compositor to present the window
            window.present();
        }
    }

    impl GtkApplicationImpl for IPlanApplication {}
    impl AdwApplicationImpl for IPlanApplication {}
}

glib::wrapper! {
    pub struct IPlanApplication(ObjectSubclass<imp::IPlanApplication>)
        @extends gio::Application, gtk::Application, adw::Application,
        @implements gio::ActionGroup, gio::ActionMap;
}

impl IPlanApplication {
    pub fn new(application_id: &str, flags: &gio::ApplicationFlags) -> Self {
        glib::Object::new(&[("application-id", &application_id), ("flags", flags)])
    }

    fn setup_gactions(&self) {
        let quit_action = gio::ActionEntry::builder("quit")
            .activate(move |app: &Self, _, _| app.quit())
            .build();
        let about_action = gio::ActionEntry::builder("about")
            .activate(move |app: &Self, _, _| app.show_about())
            .build();
        let shortcuts_action = gio::ActionEntry::builder("shortcuts")
            .activate(move |app: &Self, _, _| app.show_shortcuts())
            .build();
        let search_action = gio::ActionEntry::builder("search")
            .activate(move |app: &Self, _, _| app.show_search())
            .build();
        self.add_action_entries([quit_action, about_action, shortcuts_action, search_action])
            .unwrap();
    }

    fn show_search(&self) {
        let window = SearchWindow::new(
            self.upcast_ref::<gtk::Application>(),
            &self.active_window().unwrap(),
        );
        window.present();
    }

    fn show_shortcuts(&self) {
        let active_window = self.active_window().unwrap();
        let shortcuts_window: Option<gtk::ShortcutsWindow> =
            gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/shortcuts_window.ui")
                .object("shortcuts_window");
        if let Some(shortcuts_window) = shortcuts_window {
            shortcuts_window.set_transient_for(Some(&active_window));
            shortcuts_window.present();
        }
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("IPlan")
            .application_icon("ir.imansalmani.IPlan")
            .developer_name("Iman Salmani")
            .version(VERSION)
            .developers(vec!["Iman Salmani".into()])
            .copyright("© 2023 Iman Salmani")
            .license_type(gtk::License::Lgpl30)
            .website("https://github.com/iman-salmani/iplan")
            .issue_url("https://github.com/iman-salmani/iplan/issues/new/choose")
            // Translators: Replace "translator-credits" with your names, one name per line
            .translator_credits(&gettext("translator-credits"))
            .build();

        about.present();
    }
}
