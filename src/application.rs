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
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
use ashpd::{desktop::background::Background, WindowIdentifier};
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::{gio, glib};
use std::cell::RefCell;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::{APPLICATION_ID, VERSION};
use crate::db::models::Reminder;
use crate::db::operations::{read_reminder, read_reminders, read_task, update_reminder};
use crate::views::search::SearchWindow;
use crate::views::{BackupWindow, IPlanWindow};

mod imp {
    use super::*;

    #[derive(Debug)]
    pub struct IPlanApplication {
        pub background_hold: RefCell<Option<ApplicationHoldGuard>>,
        pub settings: gio::Settings,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IPlanApplication {
        const NAME: &'static str = "IPlanApplication";
        type Type = super::IPlanApplication;
        type ParentType = adw::Application;

        fn new() -> Self {
            Self {
                background_hold: RefCell::default(),
                settings: gio::Settings::new("ir.imansalmani.IPlan.State"),
            }
        }
    }

    impl ObjectImpl for IPlanApplication {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_gactions();
            obj.setup_settings();
            obj.set_accels_for_action("app.quit", &["<primary>q"]);
            obj.set_accels_for_action("app.shortcuts", &["<primary>question"]);
            obj.set_accels_for_action("app.search", &["<primary>f"]);
        }
    }

    impl ApplicationImpl for IPlanApplication {
        fn startup(&self) {
            self.parent_startup();
            let application = self.obj();

            #[cfg(any(target_os = "linux", target_os = "freebsd"))]
            application.request_background();

            let reminders = read_reminders(None).unwrap();
            for reminder in reminders {
                application.send_reminder(reminder);
            }
        }

        fn activate(&self) {
            let application = self.obj();
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
        glib::Object::builder()
            .property("application-id", application_id)
            .property("flags", flags)
            .build()
    }

    fn setup_settings(&self) {
        self.imp().settings.connect_changed(
            Some("background-play"),
            glib::clone!(@weak self as this => move |settings, _| {
                let background_play = settings.boolean("background-play");
                if background_play {
                    this.request_background();
                } else {
                    this.imp().background_hold.replace(None);
                }
            }),
        );

        let _dummy = self.imp().settings.boolean("background-play");
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
        let backup_action = gio::ActionEntry::builder("backup")
            .activate(move |app: &Self, _, _| app.show_backup())
            .build();
        self.add_action_entries([
            quit_action,
            about_action,
            shortcuts_action,
            search_action,
            backup_action,
        ]);
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

    fn show_backup(&self) {
        let active_window = self.active_window().unwrap();
        let backup_window = BackupWindow::new(self, &active_window);
        backup_window.present();
    }

    fn show_about(&self) {
        let window = self.active_window().unwrap();
        let about = adw::AboutWindow::builder()
            .transient_for(&window)
            .application_name("IPlan")
            .application_icon("ir.imansalmani.IPlan")
            .developer_name("Iman Salmani")
            .version(VERSION)
            .developers(vec!["Iman Salmani https://github.com/iman-salmani"])
            .copyright("© 2023 Iman Salmani")
            .license_type(gtk::License::Lgpl30)
            .website("https://github.com/iman-salmani/iplan")
            .issue_url("https://github.com/iman-salmani/iplan/issues/new/choose")
            // Translators: Replace "translator-credits" with your names, one name per line
            .translator_credits(gettext("translator-credits"))
            .build();

        about.present();
    }

    pub fn send_reminder(&self, reminder: Reminder) {
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let datetime = reminder.datetime_duration();

        if datetime < now {
            reminder.set_past(true);
            update_reminder(&reminder).unwrap();
            return;
        }

        let remains = datetime - now;
        thread::spawn(move || {
            thread::sleep(remains);
            if tx.send("").is_err() {}
        });
        rx.attach(None,glib::clone!(@weak self as obj => @default-return glib::Continue(false), move |_: &str| {
            let fresh_reminder = read_reminder(reminder.id()).expect("Failed to read reminder");

            if fresh_reminder.past() || fresh_reminder.datetime() != reminder.datetime() {
                return glib::Continue(false);
            }

            let task = read_task(fresh_reminder.task()).expect("Failed to read task");
            let notification = gio::Notification::new(&task.name());
            notification.set_priority(gio::NotificationPriority::High);
            obj.send_notification(Some(&format!("reminder-{}", fresh_reminder.id())), &notification);
            fresh_reminder.set_past(true);
            update_reminder(&fresh_reminder).expect("Failed to update reminder");

            glib::Continue(false)
        }));
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    async fn portal_request_background(&self) {
        let mut request: ashpd::desktop::background::BackgroundRequest = Background::request()
            .reason(Some(
                gettext("IPlan needs to run in the background to send reminders").as_str(),
            ))
            .auto_start(true)
            .command(&["iplan", "--gapplication-service"]);

        if let Some(window) = self.active_window() {
            let root = window.native().unwrap();
            let identifier = WindowIdentifier::from_native(&root).await;
            request = request.identifier(identifier);
        }

        match request.send().await.and_then(|r| r.response()) {
            Ok(_) => {
                self.imp().background_hold.replace(Some(self.hold()));
            }
            Err(_) => {
                self.imp()
                    .settings
                    .set_boolean("background-play", false)
                    .expect("Unable to set background-play settings key");
            }
        }
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    fn request_background(&self) {
        let ctx = glib::MainContext::default();
        ctx.spawn_local(glib::clone!(@weak self as app => async move {
            app.portal_request_background().await
        }));
    }
}
