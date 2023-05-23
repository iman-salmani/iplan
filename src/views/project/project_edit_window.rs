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
        pub icon_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub description_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub description_buffer: TemplateChild<gtk::TextBuffer>,
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
        imp.icon_label.set_text(&project.icon());
        imp.name_entry_row.set_text(&project.name());
        let task_description = project.description();
        imp.description_expander_row
            .set_subtitle(&win.description_display(&task_description));
        imp.description_buffer.set_text(&task_description);
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

    fn description_display(&self, text: &str) -> String {
        if let Some(first_line) = text.lines().next() {
            return String::from(first_line);
        }
        String::from("")
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
    fn handle_project_emoji_picked(&self, emoji: &str, _: gtk::EmojiChooser) {
        self.imp().icon_label.set_text(emoji);
        let project = self.imp().project.borrow();
        project.set_property("icon", emoji.to_string());
        update_project(&project).expect("Failed to update project");
        self.transient_for()
            .unwrap()
            .activate_action("project.update", None)
            .expect("Failed to send project.update action");
    }

    #[template_callback]
    fn handle_description_buffer_changed(&self, buffer: gtk::TextBuffer) {
        let imp: &imp::ProjectEditWindow = self.imp();
        let project = self.imp().project.take();
        let text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), true);
        imp.description_expander_row
            .set_subtitle(&self.description_display(&text));
        project.set_property("description", text);
        update_project(&project).expect("Failed to update task");
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
