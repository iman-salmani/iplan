use adw::{prelude::*, subclass::prelude::*};
use gettextrs::gettext;
use gtk::{glib, glib::once_cell::sync::Lazy};
use std::cell::RefCell;

use crate::db::models::{List, Record, Task};
use crate::db::operations::{read_task, read_tasks};
use crate::views::{project::TaskRow, project::TaskWindow, IPlanWindow};

mod imp {
    use super::*;

    #[derive(Default, gtk::CompositeTemplate)]
    #[template(resource = "/ir/imansalmani/iplan/ui/project/project_done_tasks_window.ui")]
    pub struct ProjectDoneTasksWindow {
        pub list: RefCell<List>,
        #[template_child]
        pub name_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub tasks_box: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub scrolled_window: TemplateChild<gtk::ScrolledWindow>,
        #[template_child]
        pub toast_overlay: TemplateChild<adw::ToastOverlay>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for ProjectDoneTasksWindow {
        const NAME: &'static str = "ProjectDoneTasksWindow";
        type Type = super::ProjectDoneTasksWindow;
        type ParentType = adw::Window;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
            klass.bind_template_instance_callbacks();
            klass.install_action("task.check", Some("i"), move |obj, _, value| {
                let imp = obj.imp();
                let value = value.unwrap().get().unwrap();
                let upper_row = imp.tasks_box.row_at_index(value - 1);
                let row = imp.tasks_box.row_at_index(value).unwrap();
                if let Some(upper_row) = upper_row {
                    upper_row.grab_focus();
                }
                imp.tasks_box.remove(&row);
                obj.transient_for()
                    .unwrap()
                    .activate_action("project.open", None) // TODO: just add task to list
                    .expect("Failed to activate project.open action");
            });
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for ProjectDoneTasksWindow {
        fn properties() -> &'static [glib::ParamSpec] {
            static PROPERTIES: Lazy<Vec<glib::ParamSpec>> =
                Lazy::new(|| vec![glib::ParamSpecObject::builder::<List>("list").build()]);
            PROPERTIES.as_ref()
        }
        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "list" => {
                    let value = value.get::<List>().expect("value must be a List");
                    self.list.replace(value);
                }
                _ => unimplemented!(),
            }
        }
        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "list" => self.list.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
    impl WidgetImpl for ProjectDoneTasksWindow {}
    impl WindowImpl for ProjectDoneTasksWindow {}
    impl AdwWindowImpl for ProjectDoneTasksWindow {}
}

glib::wrapper! {
    pub struct ProjectDoneTasksWindow(ObjectSubclass<imp::ProjectDoneTasksWindow>)
        @extends gtk::Widget, gtk::Window, adw::Window,
        @implements gtk::Buildable, gtk::Native, gtk::Root;
}

#[gtk::template_callbacks]
impl ProjectDoneTasksWindow {
    pub fn new(application: gtk::Application, app_window: &IPlanWindow, list: List) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", application)
            .property("list", list)
            .build();
        win.set_transient_for(Some(app_window));
        let imp = win.imp();
        let list: List = win.property("list");
        imp.name_label.set_label(&gettext("Done Tasks"));
        for task in read_tasks(list.project(), Some(list.id()), Some(true), Some(0))
            .expect("Failed to read tasks")
        {
            let project_list_task = TaskRow::new(task);
            imp.tasks_box.append(&project_list_task);
            project_list_task.init_widgets();
        }
        imp.tasks_box.set_sort_func(|row1, row2| {
            let row1_p = row1.property::<Task>("task").position();
            let row2_p = row2.property::<Task>("task").position();

            if row1_p < row2_p {
                gtk::Ordering::Larger
            } else {
                gtk::Ordering::Smaller
            }
        });

        imp.tasks_box.set_filter_func(glib::clone!(
        @weak imp => @default-return false,
        move |row| {
            let row = row.downcast_ref::<TaskRow>().unwrap();
            if row.task().suspended() {
                false
            } else {
                !row.imp().moving_out.get()
            }
        }));
        win
    }

    pub fn select_task(&self, task_id: i64) {
        let imp = self.imp();
        let tasks = imp.tasks_box.observe_children();
        let task = read_task(task_id).expect("Failed to read task");
        for i in 0..tasks.n_items() - 1 {
            if let Some(project_list_task) = tasks.item(i).and_downcast::<TaskRow>() {
                let list_task = project_list_task.task();
                if list_task.position() == task.position() as i32 {
                    project_list_task.grab_focus();
                    break;
                }
            }
        }
    }

    #[template_callback]
    fn handle_tasks_box_row_activated(&self, row: gtk::ListBoxRow, _tasks_box: gtk::ListBox) {
        let win = self.root().and_downcast::<gtk::Window>().unwrap();
        let row = row.downcast::<TaskRow>().unwrap();
        let modal = TaskWindow::new(&win.application().unwrap(), &win, row.task());
        modal.present();
        modal.connect_close_request(glib::clone!(
            @weak self as win, @weak row => @default-return gtk::Inhibit(false),
            move |_| {
                let task = read_task(row.task().id()).expect("Failed to read the task");
                let win_imp = win.imp();
                let row_imp = row.imp();
                let main_window = win.transient_for().unwrap();
                if !task.done() {
                    win_imp.tasks_box.remove(&row);
                    main_window.activate_action("project.open", None) // TODO: just add task to list (consider the task duration could be changed)
                        .expect("Failed to activate project.open action");
                } else {
                    if let Some(duration) = task.duration() {
                        row_imp.timer_button_content.set_label(&Record::duration_display(duration));
                    }
                    let task_name = task.name();
                    row_imp.name_button
                        .child()
                        .unwrap()
                        .downcast::<gtk::Label>()
                        .unwrap()
                        .set_text(&task_name);
                    row_imp.name_button.set_tooltip_text(Some(&task_name));
                    row_imp.name_entry.buffer().set_text(&task_name);
                    row_imp.task.replace(task);
                    row.changed();
                    main_window.activate_action("project.update", None).expect("Failed to send project.update signal");
                }
                gtk::Inhibit(false)
            }
        ));
    }
}
