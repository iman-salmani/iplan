import gi
from gi.repository import Gtk, GLib, Gio, Adw

from iplan.db.operations.project import read_projects
from iplan.db.operations.list import create_list, read_lists
from iplan.db.models.task import Task
from iplan.db.operations.task import read_task
from iplan.views.project.project_list import ProjectList
from iplan.views.project.project_list_task import ProjectListTask

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_lists.ui")
class ProjectLists(Gtk.ScrolledWindow):
    __gtype_name__ = "ProjectLists"
    lists_box: Gtk.Box = Gtk.Template.Child()
    placeholder = Gtk.Template.Child()
    shift_modifier = False
    shift_controller = None

    def __init__(self):
        super().__init__()

        # listen to shift press
        # used for hscroll
        # add if board layout
        self.shift_controller = Gtk.EventControllerKey()
        self.shift_controller.connect("key-pressed", self.on_key_pressed)
        self.shift_controller.connect("key-released", self.on_key_released)

        self.connect("map", self.on_mapped)

    # Actions
    def on_mapped(self, *args):
        self.disconnect_by_func(self.on_mapped)
        actions = self.props.root.props.application.actions
        actions["open_project"].connect(
            "activate",
            self.open_project
        )
        actions["new_list"].connect("activate", self.on_new_list)

        # open first project
        projects = read_projects()
        if not projects:
           self.props.root.props.application.project = list(read_projects(archive=True))[0]
        self.props.root.props.application.project = list(projects)[0]
        self.activate_action("app.open_project", GLib.Variant("i", -1))

    def on_key_pressed(self, controller, keyval, keycode, state):
        if keycode == 50:
            self.shift_modifier = True

    def on_key_released(self, controller, keyval, keycode, state):
        if keycode == 50:
            self.shift_modifier = False

    def set_layout(self, layout):
        if layout == "horizontal":
            self.lists_box.set_orientation(Gtk.Orientation.HORIZONTAL)
            for _list in self.lists_box.observe_children():
                if type(_list) == Adw.StatusPage:    # Checking placeholder
                    break
                _list.tasks_box.unparent()
                _list.scrolled_window.set_child(_list.tasks_box)
                _list.scrolled_window.set_visible(True)
            self.get_root().add_controller(self.shift_controller)
        else:
            self.lists_box.set_orientation(Gtk.Orientation.VERTICAL)
            for _list in self.lists_box.observe_children():
                if type(_list) == Adw.StatusPage:    # Checking placeholder
                    break
                _list.scrolled_window.set_visible(False)
                _list.tasks_box.unparent()
                _list.append(_list.tasks_box)
            self.get_root().remove_controller(self.shift_controller)

    def on_new_list(self, *args):
        _list = create_list(
            "New List",
            self.props.root.props.application.project._id
        )
        list_ui = ProjectList(_list)
        if self.placeholder.get_parent():
            self.lists_box.remove(self.placeholder)
        self.lists_box.append(list_ui)
        list_ui.name_button.set_visible(False)  # name entry visiblity have binding to this
        GLib.idle_add(lambda *args: self.get_root().set_focus(list_ui.name_entry))

    def open_project(self, action: Gio.SimpleAction, param: GLib.Variant):
        task_id = param.unpack()

        self.clear()

        if task_id != -1:
            self.fetch(read_task(task_id))
        else:
            self.fetch()

    # UI
    def clear(self):
        while True:
            row = self.lists_box.get_first_child()
            if row:
                self.lists_box.remove(row)
            else:
                break

    def fetch(self, target_task: Task=None):
        lists = read_lists(self.props.root.props.application.project._id)
        for _list in lists:
            list_ui = ProjectList(_list)

            self.lists_box.append(list_ui)

            if target_task:
                if _list._id == target_task._list:
                    list_ui.focus_on_task(target_task)

        if not self.lists_box.get_first_child():
            self.lists_box.append(self.placeholder)
            return

        if not target_task:
            first_list = self.lists_box.get_first_child()
            if first_list:
                first_row = first_list.tasks_box.get_first_child()
                if first_row:
                    GLib.idle_add(lambda *args: self.get_root().set_focus(first_row))

