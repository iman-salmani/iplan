use gettextrs::gettext;
use gtk::glib::Properties;
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::Duration;

use crate::db::models::{Project, Task};
use crate::db::operations::{create_section, read_section, read_sections, read_task};
use crate::views::project::{ProjectHeader, SectionBox};
use crate::views::{task::TaskRow, ActionScope, IPlanWindow};

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub enum ProjectLayout {
    Horizontal,
    #[default]
    Vertical,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_page.ui")]
    #[properties(wrapper_type=super::ProjectPage)]
    pub struct ProjectPage {
        pub layout: Cell<ProjectLayout>,
        #[property(get, set)]
        pub project: RefCell<Project>,
        #[property(get, set)]
        pub drag_scroll_controller: RefCell<Option<gtk::DropTarget>>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
        #[template_child]
        pub page_header: TemplateChild<adw::HeaderBar>,
        #[template_child]
        pub toggle_sidebar_button: TemplateChild<gtk::ToggleButton>,
        #[template_child]
        pub project_header: TemplateChild<ProjectHeader>,
        #[template_child]
        pub layout_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub sections_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Box>,
        #[template_child]
        pub placeholder_subtitle_start: TemplateChild<gtk::Label>,
        #[template_child]
        pub placeholder_subtitle_end: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectPage {
        const NAME: &'static str = "ProjectPage";
        type Type = super::ProjectPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action("hscroll", Some("x"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get::<f64>().unwrap();
                let adjustment = imp.scrolled_window.hadjustment();
                adjustment.set_value(adjustment.value() + (adjustment.step_increment() * value));
            });
            klass.install_action(
                "task.duration-changed",
                Some(Task::static_variant_type().as_str()),
                move |obj, _, value| {
                    let task: Task = value.unwrap().get().unwrap();
                    obj.parent()
                        .unwrap()
                        .activate_action(
                            "task.duration-changed",
                            Some(&glib::Variant::from((
                                task.to_variant(),
                                ActionScope::Project(task.project()).to_variant(),
                            ))),
                        )
                        .unwrap();
                    obj.imp().project_header.set_stat_updated(false);
                },
            );
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectPage {
        fn constructed(&self) {
            // Translators: {} Will be replaced with a shortcut label.
            let placeholder_subtitle = gettext("Use the primary menu {} for adding a new section");
            let placeholder_subtitle = placeholder_subtitle.split_once("{}").unwrap();
            self.placeholder_subtitle_start
                .set_label(placeholder_subtitle.0);
            self.placeholder_subtitle_end
                .set_label(placeholder_subtitle.1);
        }

        fn dispose(&self) {
            self.obj().first_child().unwrap().unparent();
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
    impl WidgetImpl for ProjectPage {}
    impl BoxImpl for ProjectPage {}
}

glib::wrapper! {
    pub struct ProjectPage(ObjectSubclass<imp::ProjectPage>)
        @extends glib::InitiallyUnowned, gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for ProjectPage {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl ProjectPage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_project(&self, project: &Project) {
        let imp = self.imp();
        let project_id = project.id();

        imp.project_header.open_project(project);

        let section_boxes = imp.sections_box.observe_children();
        for _ in 0..section_boxes.n_items() {
            imp.sections_box
                .remove(&section_boxes.item(0).and_downcast::<gtk::Widget>().unwrap());
        }

        let sections = read_sections(project_id).unwrap();
        if sections.is_empty() {
            imp.sections_box.append(&imp.placeholder.get());
        } else {
            let mut max_height = self.height();
            if max_height == 0 {
                max_height = self
                    .root()
                    .and_downcast::<IPlanWindow>()
                    .unwrap()
                    .default_height();
            }
            for section in sections {
                let section_box = SectionBox::new(section, imp.layout.get(), max_height as usize);
                imp.sections_box.append(&section_box);
            }
        }
        self.set_project(project);
    }

    pub fn reset_task(&self, mut task: Task) {
        let task_parent = task.parent();
        let is_subtask = task_parent != 0;
        if is_subtask {
            if let Ok(parent) = read_task(task_parent) {
                // FIXME: find better way instead of read_task
                task = parent;
            } else {
                return;
            }
        }

        if let Some(section_box) = self.section_by_id(task.section()) {
            let tasks_box = section_box.imp().tasks_box.get();
            if let Some(row) = tasks_box.item_by_id(task.id()) {
                if is_subtask {
                    row.reset_subtasks();
                } else if task.done() {
                    tasks_box.remove_item(&row);
                } else {
                    row.reset(task);
                    row.changed();
                }
            } else if !task.done() {
                let row = TaskRow::new(task, false, false);
                tasks_box.add_item(&row);
            }
        }
    }

    pub fn refresh_task_timer(&self, mut task: Task) {
        while task.parent() != 0 {
            task = read_task(task.parent()).unwrap();
        }

        if let Some(row) = self.task_row(&task) {
            row.refresh_timer();
        }
    }

    pub fn task_row(&self, task: &Task) -> Option<TaskRow> {
        if let Some(section_box) = self.section_by_id(task.section()) {
            let tasks_box = section_box.imp().tasks_box.get();
            if let Some(row) = tasks_box.item_by_id(task.id()) {
                return Some(row);
            }
        }
        None
    }

    pub fn select_task(&self, task_id: Option<i64>) {
        let imp = self.imp();
        if let Some(task_id) = task_id {
            let mut task = read_task(task_id).unwrap();
            if task.parent() != 0 {
                task = read_task(task.parent()).unwrap();
            }
            let section = read_section(task.section()).unwrap();
            let section_box = imp
                .sections_box
                .observe_children()
                .item(section.index() as u32)
                .and_downcast::<SectionBox>()
                .unwrap();
            section_box.select_task(task);
        } else if let Some(first_section_box) =
            imp.sections_box.first_child().and_downcast::<SectionBox>()
        {
            if let Some(first_row) = first_section_box
                .imp()
                .tasks_box
                .first_child()
                .and_downcast::<TaskRow>()
            {
                first_row.grab_focus();
            }
        }
    }

    pub fn new_section(&self, project_id: i64) {
        let imp = self.imp();
        let section = create_section(&gettext("New Section"), project_id).unwrap();
        let section_box = SectionBox::new(section, imp.layout.get(), 18);
        if imp.placeholder.parent().is_some() {
            imp.sections_box.remove(&imp.placeholder.get());
        }
        imp.sections_box.append(&section_box);
        let section_box_imp = section_box.imp();
        section_box_imp.name_button.set_visible(false); // Name entry visibility have binding to this
        let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        glib::idle_add_once(move || tx.send("").unwrap());
        let name_entry = section_box_imp.name_entry.get();
        rx.attach(None, move |_text| {
            name_entry.grab_focus();
            glib::ControlFlow::Break
        });
    }

    pub fn set_layout(&self, layout: ProjectLayout) {
        let imp = self.imp();
        match layout {
            ProjectLayout::Horizontal => {
                imp.sections_box
                    .set_orientation(gtk::Orientation::Horizontal);
                if let Some(controller) = self.drag_scroll_controller() {
                    self.remove_controller(&controller);
                }
                self.add_drag_hscroll_controller();
            }
            ProjectLayout::Vertical => {
                imp.sections_box.set_orientation(gtk::Orientation::Vertical);
                if let Some(controller) = self.drag_scroll_controller() {
                    self.remove_controller(&controller);
                }
                self.add_drag_vscroll_controller();
            }
        }
        let section_boxes = imp.sections_box.observe_children();
        for i in 0..section_boxes.n_items() {
            let section_box = section_boxes.item(i).and_downcast::<SectionBox>().unwrap();
            section_box.set_layout(&layout);
        }
        imp.layout.set(layout);
    }

    fn section_by_id(&self, id: i64) -> Option<SectionBox> {
        let sections_box = self.imp().sections_box.observe_children();
        for i in 0..sections_box.n_items() {
            let item = sections_box.item(i).and_downcast::<SectionBox>();
            if let Some(item) = item {
                if item.section().id() == id {
                    return Some(item);
                }
            }
        }
        None
    }

    fn add_drag_hscroll_controller(&self) {
        let controller = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        controller.set_preload(true);

        controller.connect_motion(|controller, x, _| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            let width = obj.width();

            if width - (x as i32) < 50 {
                if obj.scroll() != 2 {
                    obj.set_scroll(2);
                    obj.start_scroll();
                }
            } else if x < 50.0 {
                if obj.scroll() != -2 {
                    obj.set_scroll(-2);
                    obj.start_scroll();
                }
            } else {
                obj.set_scroll(0)
            }

            gdk::DragAction::empty()
        });
        controller.connect_leave(|controller| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            obj.set_scroll(0);
        });
        self.set_drag_scroll_controller(controller.clone());
        self.add_controller(controller);
    }

    fn add_drag_vscroll_controller(&self) {
        let controller = gtk::DropTarget::new(TaskRow::static_type(), gdk::DragAction::MOVE);
        controller.set_preload(true);

        controller.connect_motion(|controller, _, y| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            let height = obj.height();

            if height - (y as i32) < 50 {
                if obj.scroll() != 1 {
                    obj.set_scroll(1);
                    obj.start_scroll();
                }
            } else if y < 50.0 {
                if obj.scroll() != -1 {
                    obj.set_scroll(-1);
                    obj.start_scroll();
                }
            } else {
                obj.set_scroll(0)
            }

            gdk::DragAction::empty()
        });
        controller.connect_leave(|controller| {
            let obj = controller.widget().downcast::<Self>().unwrap();
            obj.set_scroll(0);
        });
        self.set_drag_scroll_controller(controller.clone());
        self.add_controller(controller);
    }

    fn start_scroll(&self) {
        let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);
        thread::spawn(move || loop {
            if tx.send(()).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.1));
        });
        rx.attach(
            None,
            glib::clone!(@weak self as obj => @default-return glib::ControlFlow::Break, move |_| {
                let scroll = obj.scroll();
                match scroll {
                    0 => glib::ControlFlow::Break,
                    1 => {
                        obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepDown, false);
                        glib::ControlFlow::Continue
                    }
                    -1 => {
                        obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepUp, false);
                        glib::ControlFlow::Continue
					}
                    2 => {
                        obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepRight, false);
                        glib::ControlFlow::Continue
					}
                    -2 => {
                        obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepLeft, false);
                        glib::ControlFlow::Continue
					}
                    _ => glib::ControlFlow::Break,
                }
            }),
        );
    }
}
