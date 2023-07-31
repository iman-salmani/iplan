use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::glib;
use gtk::glib::Properties;
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{delete_project, update_project};
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_edit_window.ui")]
    #[properties(type_wrapper=super::ProjectEditWindow)]
    pub struct ProjectEditWindow {
        #[property(get, set)]
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
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectEditWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("changed")
                    .param_types([Project::static_type()])
                    .build()]
            });
            SIGNALS.as_ref()
        }

        fn properties() -> &'static [glib::ParamSpec] {
            Self::derived_properties()
        }

        fn property(&self, id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            self.derived_property(id, pspec)
        }

        fn set_property(&self, id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            self.derived_set_property(id, value, pspec)
        }
    }
    impl WidgetImpl for ProjectEditWindow {}
    impl WindowImpl for ProjectEditWindow {}
}

glib::wrapper! {
    pub struct ProjectEditWindow(ObjectSubclass<imp::ProjectEditWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl ProjectEditWindow {
    pub fn new(application: gtk::Application, app_window: &IPlanWindow, project: Project) -> Self {
        let obj: Self = glib::Object::builder()
            .property("application", application)
            .build();
        obj.set_transient_for(Some(app_window));
        let imp = obj.imp();

        imp.icon_label.set_text(&project.icon());

        imp.name_entry_row.set_text(&project.name());

        obj.set_project(project);
        obj.add_bindings();
        obj
    }

    fn add_bindings(&self) {
        let imp = self.imp();
        let project = self.project();

        project
            .bind_property("archive", &imp.archive_switch.get(), "active")
            .sync_create()
            .bidirectional()
            .build();

        project
            .bind_property("description", &imp.description_buffer.get(), "text")
            .sync_create()
            .bidirectional()
            .build();

        project.connect_notify_local(
            None,
            glib::clone!(@weak self as obj => move|project, _| {
                update_project(project).expect("Failed to update task");
                obj.emit_by_name::<()>("changed", &[&project]);
            }),
        );

        imp.description_buffer
            .bind_property("text", &imp.description_expander_row.get(), "subtitle")
            .transform_to(|_, text: String| text.lines().next().map(String::from))
            .sync_create()
            .build();
    }

    #[template_callback]
    fn handle_name_entry_row_apply(&self, entry_row: adw::EntryRow) {
        let project = self.project();
        project.set_name(entry_row.text());
    }

    #[template_callback]
    fn handle_project_emoji_picked(&self, emoji: &str, _: gtk::EmojiChooser) {
        let project = self.project();
        self.imp().icon_label.set_text(emoji);
        project.set_icon(emoji.to_string());
    }

    #[template_callback]
    fn handle_delete_button_clicked(&self, _button: gtk::Button) {
        let dialog =
            gtk::Builder::from_resource("/ir/imansalmani/iplan/ui/snippets/delete_dialog.ui")
                .object::<adw::MessageDialog>("dialog")
                .unwrap();
        dialog.set_transient_for(self.transient_for().as_ref());
        let project = self.project();
        let dialog_heading = gettext("Delete “{}” project?");
        dialog.set_heading(Some(&dialog_heading.replace("{}", &project.name())));
        dialog.set_body(&gettext(
            "The project and its tasks will be permanently lost.",
        ));
        dialog.connect_response(Some("delete"), move |dialog, response| {
            if response == "delete" {
                delete_project(project.id(), project.index()).unwrap();
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
