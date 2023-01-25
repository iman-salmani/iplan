import gi

from gi.repository import Gtk, GLib, Gio

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

    def __init__(self):
        super().__init__()

        self.connect("map", self.on_mapped)

    # Actions
    def on_mapped(self, *args):
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

    def on_new_list(self, *args):
        _list = create_list(
            "New List",
            self.props.root.props.application.project._id
        )
        self.lists_box.append(ProjectList(_list))

    def open_project(self, action: Gio.SimpleAction, param: GLib.Variant):
        # TODO: do unpack on other instances
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

