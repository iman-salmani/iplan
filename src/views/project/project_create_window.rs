use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use glib::{once_cell::sync::Lazy, subclass::Signal};
use gtk::glib;
use gtk::glib::Properties;
use std::cell::RefCell;

use crate::db::models::Project;
use crate::db::operations::{create_project, create_section};
use crate::views::IPlanWindow;

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_create_window.ui")]
    #[properties(type_wrapper=super::ProjectCreateWindow)]
    pub struct ProjectCreateWindow {
        #[property(get, set)]
        pub project: RefCell<Project>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
        #[template_child]
        pub icon_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub name_entry_row: TemplateChild<adw::EntryRow>,
        #[template_child]
        pub description_expander_row: TemplateChild<adw::ExpanderRow>,
        #[template_child]
        pub description_buffer: TemplateChild<gtk::TextBuffer>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectCreateWindow {
        const NAME: &'static str = "ProjectCreateWindow";
        type Type = super::ProjectCreateWindow;
        type ParentType = gtk::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectCreateWindow {
        fn signals() -> &'static [glib::subclass::Signal] {
            static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
                vec![Signal::builder("project-created")
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
    impl WidgetImpl for ProjectCreateWindow {}
    impl WindowImpl for ProjectCreateWindow {}
}

glib::wrapper! {
    pub struct ProjectCreateWindow(ObjectSubclass<imp::ProjectCreateWindow>)
        @extends gtk::Widget, gtk::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl ProjectCreateWindow {
    pub fn new(application: &gtk::Application, app_window: &IPlanWindow) -> Self {
        let obj: Self = glib::Object::builder()
            .property("application", application)
            .build();
        obj.set_transient_for(Some(app_window));
        obj.imp().name_entry_row.grab_focus();
        obj.add_bindings();
        obj
    }

    fn add_bindings(&self) {
        let imp = self.imp();
        let project = self.project();

        project
            .bind_property("description", &imp.description_buffer.get(), "text")
            .sync_create()
            .bidirectional()
            .build();

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
    fn handle_cancel_button_clicked(&self, _: gtk::Button) {
        self.close();
    }

    #[template_callback]
    fn handle_add_button_clicked(&self, _: gtk::Button) {
        let project = self.project();

        if project.name().trim() == "" {
            let toast = adw::Toast::builder()
                .title(gettext("Project should have a name!"))
                .build();
            self.imp().toast_overlay.add_toast(toast);
            return;
        }

        let project =
            create_project(&project.name(), &project.icon(), &project.description()).unwrap();
        create_section(&gettext("Tasks"), project.id()).unwrap();
        self.emit_by_name::<()>("project-created", &[&project]);
        self.close()
    }
}
