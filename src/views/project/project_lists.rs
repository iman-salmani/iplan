use gettextrs::gettext;
use gtk::glib::Properties;
use gtk::{gdk, glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};
use std::thread;
use std::time::Duration;

use crate::db::operations::{create_list, read_list, read_lists, read_task};
use crate::views::project::{ProjectList, TaskRow};
use crate::views::IPlanWindow;

#[derive(Default, Clone, Copy, PartialEq)]
pub enum ProjectLayout {
    Horizontal,
    #[default]
    Vertical,
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate, Properties)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_lists.ui")]
    #[properties(wrapper_type=super::ProjectLists)]
    pub struct ProjectLists {
        pub layout: Cell<ProjectLayout>,
        #[property(get, set)]
        pub drag_scroll_controller: RefCell<Option<gtk::DropTarget>>,
        #[property(get, set)]
        pub scroll: Cell<i8>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub lists_box: TemplateChild<gtk::Box>,
        #[template_child]
        pub placeholder: TemplateChild<gtk::Box>,
        #[template_child]
        pub placeholder_subtitle_start: TemplateChild<gtk::Label>,
        #[template_child]
        pub placeholder_subtitle_end: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectLists {
        const NAME: &'static str = "ProjectLists";
        type Type = super::ProjectLists;
        type ParentType = gtk::Widget;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.install_action("hscroll", Some("x"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get::<f64>().unwrap();
                let adjustment = imp.scrolled_window.hadjustment();
                adjustment.set_value(adjustment.value() + (adjustment.step_increment() * value));
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectLists {
        fn constructed(&self) {
            // Translators: {} Will be replaced with a shortcut label.
            let placeholder_subtitle = gettext("Use the primary menu {} for adding new lists");
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
    impl BuildableImpl for ProjectLists {}
    impl WidgetImpl for ProjectLists {
        fn request_mode(&self) -> gtk::SizeRequestMode {
            self.parent_request_mode();
            gtk::SizeRequestMode::ConstantSize
        }

        fn measure(&self, orientation: gtk::Orientation, for_size: i32) -> (i32, i32, i32, i32) {
            self.obj()
                .first_child()
                .unwrap()
                .measure(orientation, for_size)
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.obj()
                .first_child()
                .unwrap()
                .size_allocate(&gtk::Allocation::new(0, 0, width, height), baseline);
        }
    }
}

glib::wrapper! {
    pub struct ProjectLists(ObjectSubclass<imp::ProjectLists>)
        @extends glib::InitiallyUnowned, gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for ProjectLists {
    fn default() -> Self {
        glib::Object::builder().build()
    }
}

impl ProjectLists {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open_project(&self, project_id: i64) {
        let imp = self.imp();

        let lists = imp.lists_box.observe_children();
        for _ in 0..lists.n_items() {
            imp.lists_box
                .remove(&lists.item(0).and_downcast::<gtk::Widget>().unwrap());
        }

        let lists = read_lists(project_id).unwrap();
        if lists.is_empty() {
            imp.lists_box.append(&imp.placeholder.get());
        } else {
            let mut max_height = self.height();
            if max_height == 0 {
                max_height = self
                    .root()
                    .and_downcast::<IPlanWindow>()
                    .unwrap()
                    .default_height();
            }
            for list in lists {
                let project_list = ProjectList::new(list, imp.layout.get(), max_height as usize);
                imp.lists_box.append(&project_list);
            }
        }
    }

    pub fn select_task(&self, task_id: Option<i64>) {
        let imp = self.imp();
        if let Some(task_id) = task_id {
            let mut task = read_task(task_id).expect("Failed to read task");
            if task.parent() != 0 {
                task = read_task(task.parent()).expect("Failed to read task")
            }
            let list = read_list(task.list()).expect("Failed to read list");
            let project_list = imp
                .lists_box
                .observe_children()
                .item(list.index() as u32)
                .and_downcast::<ProjectList>()
                .unwrap();
            project_list.select_task(task);
        } else if let Some(first_list) = imp.lists_box.first_child().and_downcast::<ProjectList>() {
            if let Some(first_row) = first_list
                .imp()
                .tasks_box
                .first_child()
                .and_downcast::<TaskRow>()
            {
                first_row.grab_focus();
            }
        }
    }

    pub fn new_list(&self, project_id: i64) {
        let imp = self.imp();
        let list =
            create_list(&gettext("New List"), project_id).expect("Failed to create new list");
        let project_list = ProjectList::new(list, imp.layout.get(), 18);
        if imp.placeholder.parent().is_some() {
            imp.lists_box.remove(&imp.placeholder.get());
        }
        imp.lists_box.append(&project_list);
        let project_list_imp = project_list.imp();
        project_list_imp.name_button.set_visible(false); // Name entry visibility have binding to this
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        glib::idle_add_once(move || tx.send("").unwrap());
        let name_entry = project_list_imp.name_entry.get();
        rx.attach(None, move |_text| {
            name_entry.grab_focus();
            glib::Continue(false)
        });
    }

    pub fn set_layout(&self, layout: ProjectLayout) {
        let imp = self.imp();
        match layout {
            ProjectLayout::Horizontal => {
                imp.lists_box.set_orientation(gtk::Orientation::Horizontal);
                if let Some(controller) = self.drag_scroll_controller() {
                    self.remove_controller(&controller);
                }
            }
            ProjectLayout::Vertical => {
                imp.lists_box.set_orientation(gtk::Orientation::Vertical);
                if let Some(controller) = self.drag_scroll_controller() {
                    self.add_controller(controller);
                } else {
                    self.add_drag_scroll_controller();
                }
            }
        }
        imp.layout.set(layout);
    }

    fn add_drag_scroll_controller(&self) {
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
        let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
        thread::spawn(move || loop {
            if tx.send(()).is_err() {
                break;
            }
            thread::sleep(Duration::from_secs_f32(0.1));
        });
        rx.attach(
            None,
            glib::clone!(@weak self as obj => @default-return glib::Continue(false), move |_| {
                let scroll = obj.scroll();
                if scroll == 0 {
                    glib::Continue(false)
                } else if scroll.is_positive() {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepDown, false);
                    glib::Continue(true)
                } else {
                    obj.imp().scrolled_window.emit_scroll_child(gtk::ScrollType::StepUp, false);
                    glib::Continue(true)
                }
            }),
        );
    }
}
