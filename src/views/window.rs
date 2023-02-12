/* window.rs
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
use gtk::prelude::*;
use gtk::{gio, glib};

use crate::views::sidebar::Sidebar;

mod imp {
    use super::*;

    #[derive(gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/window.ui")]
    pub struct IPlanWindow {
        pub settings: gio::Settings,

        #[template_child]
        pub project_layout_button: TemplateChild<gtk::Button>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for IPlanWindow {
        const NAME: &'static str = "IPlanWindow";
        type Type = super::IPlanWindow;
        type ParentType = adw::ApplicationWindow;

        fn class_init(klass: &mut Self::Class) {
            Sidebar::ensure_type();
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }

        fn new() -> Self {
            Self {
                settings: gio::Settings::new("ir.imansalmani.iplan.State"),
                project_layout_button: TemplateChild::default(),
            }
        }
    }

    impl ObjectImpl for IPlanWindow {}
    impl WidgetImpl for IPlanWindow {}
    impl WindowImpl for IPlanWindow {}
    impl ApplicationWindowImpl for IPlanWindow {}
    impl AdwApplicationWindowImpl for IPlanWindow {}
}

glib::wrapper! {
    pub struct IPlanWindow(ObjectSubclass<imp::IPlanWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow,
        @implements gio::ActionGroup, gio::ActionMap;
}

#[gtk::template_callbacks]
impl IPlanWindow {
    pub fn new<P: glib::IsA<gtk::Application>>(application: &P) -> Self {
        let window = glib::Object::new::<IPlanWindow>(&[("application", application)]);

        // Set project layout
        let imp = window.imp();
        if imp.settings.int("default-project-layout") == 1 {
            imp.project_layout_button
                .set_icon_name("view-columns-symbolic")
        }

        window
    }

    #[template_callback]
    fn handle_project_layout_button_clicked(&self, button: gtk::Button) {
        let imp = self.imp();
        match button.icon_name() {
            Some(icon_name) => {
                if icon_name == "list-symbolic" {
                    button.set_icon_name("view-columns-symbolic");
                    imp.settings
                        .set_int("default-project-layout", 1)
                        .expect("Could not set setting.");
                } else {
                    button.set_icon_name("list-symbolic");
                    imp.settings
                        .set_int("default-project-layout", 0)
                        .expect("Could not set setting.");
                }
            }
            None => button.set_icon_name("list-symbolic"),
        }
    }
}
