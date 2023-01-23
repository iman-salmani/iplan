import gi
from gi.repository import Gtk, Adw, GLib

from iplan.db.models.project import Project
from iplan.db.models.task import Task
from iplan.db.operations.project import read_project

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search_bar/search_result.ui')
class SearchResult(Gtk.Button):
    __gtype_name__ = "SearchResult"
    name = Gtk.Template.Child()
    _type: str
    project: Project
    task: Task
    menu: Gtk.Popover

    def __init__(self, _type, name, menu, project=None, task=None):
        super().__init__()
        self._type = _type
        self.project = project
        self.task = task
        self.menu = menu
        self.name.set_label(name)

    @Gtk.Template.Callback()
    def on_clicked(self, sender):
        self.menu.popdown()
        if self._type == "project":
            self.props.root.props.application.project = self.project
            self.activate_action("app.open_project", GLib.Variant("i", -1))
        elif self._type == "task":
            self.props.root.props.application.project = \
                read_project(self.task.project)
            self.activate_action(
                "app.open_project",
                GLib.Variant("i", self.task._id)
            )

