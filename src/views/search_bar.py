import gi
from gi.repository import Gtk, Adw, GLib, Gio, Gdk

from iplan.database.database import ProjectsData, Project, TasksData, Task

# Initialize Database connection
projects_data = ProjectsData()
tasks_data = TasksData()

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search_bar.ui')
class SearchBar(Gtk.Box):
    __gtype_name__ = "SearchBar"
    search_entry = Gtk.Template.Child()
    menu = Gtk.Template.Child()
    founded_items = Gtk.Template.Child()

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.connect("map", lambda *args: self.install_actions())

    # Actions
    def install_actions(self):
        "Install actions after widget mapped"
        actions = self.props.root.props.application.actions
        actions["search"].connect("activate", lambda *args: self.search_entry.grab_focus())

    @Gtk.Template.Callback()
    def on_search_entry_change(self, sender):
        self.clear()

        result = [[], []]
        text = sender.get_text().lower()

        if text:
            result[0] = projects_data.search(text)
            result[1] = tasks_data.search(text)

        if result == [[], []]:
            self.menu.popdown()
        else:
            self.menu.popup()

            for project in result[0]:
                self.founded_items.append(
                    SearchItem("project", project.name, self.menu, project=project))
            for task in result[1]:
                self.founded_items.append(
                    SearchItem("task", task.name, self.menu, task=task))

    # UI
    def clear(self):
        while True:
            row = self.founded_items.get_first_child()
            if row:
                self.founded_items.remove(row)
            else:
                break


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search_item.ui')
class SearchItem(Gtk.Button):
    __gtype_name__ = "SearchItem"
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
    def on_click(self, sender):
        self.menu.popdown()
        if self._type == "project":
            self.props.root.project = self.project
            self.activate_action("app.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", -1)
            ))
        elif self._type == "task":
            self.props.root.project = projects_data.get(self.task.project)
            self.activate_action("app.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", self.task.id)
            ))
