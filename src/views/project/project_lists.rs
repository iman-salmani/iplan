use gettextrs::gettext;
use gtk::{glib, prelude::*, subclass::prelude::*};
use std::cell::{Cell, RefCell};

use crate::db::operations::{create_list, read_list, read_lists, read_task};
use crate::views::project::{ProjectList, TaskRow};
use crate::views::IPlanWindow;

#[derive(Clone, Copy, PartialEq)]
pub enum ProjectLayout {
    Horizontal,
    Vertical,
}

impl Default for ProjectLayout {
    fn default() -> Self {
        ProjectLayout::Vertical
    }
}

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_lists.ui")]
    pub struct ProjectLists {
        pub layout: Cell<ProjectLayout>,
        pub shift_pressed: Cell<bool>,
        pub shift_controller: RefCell<Option<gtk::EventControllerKey>>,
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
        @extends gtk::Widget,
        @implements gtk::Buildable;
}

impl ProjectLists {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn open_project(&self, project_id: i64) {
        let imp = self.imp();

        let lists = imp.lists_box.observe_children();
        for _i in 0..lists.n_items() {
            imp.lists_box
                .remove(&lists.item(0).and_downcast::<gtk::Widget>().unwrap());
        }

        // let tasks_per_page = self.allocated_height() / 72;   // TODO: update by window resize
        let tasks_per_page = 18;
        for list in read_lists(project_id).expect("Failed to read lists") {
            let project_list = ProjectList::new(list);
            imp.lists_box.append(&project_list);
            project_list.init_widgets(project_id, imp.layout.get(), tasks_per_page as usize);
        }
        if imp.lists_box.first_child().is_none() {
            imp.lists_box.append(&imp.placeholder.get());
        }
    }

    pub fn select_task(&self, task_id: Option<i64>) {
        let imp = self.imp();
        if let Some(task_id) = task_id {
            let task = read_task(task_id).expect("Failed to read task");
            let list = read_list(task.list()).expect("Failed to read list");
            let project_list = imp
                .lists_box
                .observe_children()
                .item(list.index() as u32)
                .and_downcast::<ProjectList>()
                .unwrap();
            project_list.select_task(task);
        } else {
            if let Some(first_list) = imp.lists_box.first_child().and_downcast::<ProjectList>() {
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
    }

    pub fn new_list(&self, project_id: i64) {
        let list =
            create_list(&gettext("New List"), project_id).expect("Failed to create new list");
        let project_list = ProjectList::new(list);
        let imp = self.imp();
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

    pub fn set_layout(&self, window: &IPlanWindow, layout: ProjectLayout) {
        let imp = self.imp();
        match layout {
            ProjectLayout::Horizontal => {
                imp.lists_box.set_orientation(gtk::Orientation::Horizontal);
                let mut shift_controller = imp.shift_controller.borrow_mut();
                if let Some(shift_controller) = shift_controller.as_ref() {
                    window.add_controller(shift_controller);
                } else {
                    let new_shift_controller = gtk::EventControllerKey::new();
                    new_shift_controller.connect_key_pressed(glib::clone!(
                        @weak self as obj => @default-return gtk::Inhibit(false),
                        move |_controller, _keyval, keycode, _state| {
                            if keycode == 50 {
                                let imp = obj.imp();
                                imp.shift_pressed.set(true);
                                let lists = imp.lists_box.observe_children();
                                for i in 0..lists.n_items() {
                                    lists.item(i)
                                        .and_downcast::<ProjectList>()
                                        .unwrap()
                                        .imp()
                                        .scrolled_window
                                        .vscrollbar()
                                        .set_sensitive(false);
                                }}
                            gtk::Inhibit(true)}));
                    new_shift_controller.connect_key_released(glib::clone!(
                    @weak self as obj =>
                    move |_controller, _keyval, keycode, _state| {
                        if keycode == 50 {
                            let imp = obj.imp();
                            imp.shift_pressed.set(false);
                            let lists = imp.lists_box.observe_children();
                            for i in 0..lists.n_items() {
                                lists.item(i)
                                    .and_downcast::<ProjectList>()
                                    .unwrap()
                                    .imp()
                                    .scrolled_window
                                    .vscrollbar()
                                    .set_sensitive(true);
                            }}}));
                    window.add_controller(&new_shift_controller);
                    shift_controller.replace(new_shift_controller);
                }
            }
            ProjectLayout::Vertical => {
                imp.lists_box.set_orientation(gtk::Orientation::Vertical);
                if let Some(shift_controller) = imp.shift_controller.borrow().as_ref() {
                    window.remove_controller(shift_controller);
                }
            }
        }
        imp.layout.set(layout);
    }
}
