use adw::{prelude::*, subclass::prelude::*};
use gtk::glib;
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{delete_project, update_project};
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_edit_window.ui")]
    pub struct ProjectEditWindow {
        pub project: RefCell<Project>,
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub archive_switch: TemplateChild<gtk::Switch>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectEditWindow {
        const NAME: &'static str = "ProjectEditWindow";
        type Type = super::ProjectEditWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectEditWindow {}
    impl WidgetImpl for ProjectEditWindow {}
    impl WindowImpl for ProjectEditWindow {}
    impl AdwWindowImpl for ProjectEditWindow {}
}

glib::wrapper! {
    pub struct ProjectEditWindow(ObjectSubclass<imp::ProjectEditWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl ProjectEditWindow {
    pub fn new(application: gtk::Application, app_window: &IPlanWindow, project: Project) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        imp.name_entry_row.set_text(&project.name());
        imp.archive_switch.set_active(project.archive());
        imp.archive_switch.connect_state_set(glib::clone!(
        @weak win, @weak project => @default-return gtk::Inhibit(true),
        move |_switch, state| {
            project.set_property("archive", state);
            win.transient_for().unwrap()
                .activate_action("project.update", None)
                .expect("Failed to send project.update action");
            update_project(&project).expect("Failed to update project");
            gtk::Inhibit(false)
        }));
        imp.project.replace(project);
        win
    }

    #[template_callback]
    fn handle_name_entry_row_apply(&self, entry_row: adw::EntryRow) {
        let project = self.imp().project.borrow();
        project.set_property("name", entry_row.text());
        update_project(&project).expect("Failed to update project");
        self.transient_for()
            .unwrap()
            .activate_action("project.update", None)
            .expect("Failed to send project.update action");
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let dialog = gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/delete_dialog.ui")
            .object::<adw::MessageDialog>("dialog")
            .unwrap();
        dialog.set_transient_for(self.transient_for().as_ref());
        let project = self.imp().project.take();
        dialog.set_heading(Some(&format!("Delete \"{}\" List?", project.name())));
        dialog.set_body("Project and tasks will be permanently lost.");
        dialog.connect_response(Some("delete"), move |dialog, response| {
            if response == "delete" {
                delete_project(project.id(), project.index()).expect("Failed to delete list");
                dialog
                    .transient_for()
                    .unwrap()
                    .activate_action("project.delete", None)
                    .expect("Failed to send project.delete action");
            }
        });
        dialog.present();
        self.close();
    }
}
