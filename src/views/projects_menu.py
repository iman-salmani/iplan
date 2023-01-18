import gi
from gi.repository import Gtk, Adw, GLib, Gio

from iplan.database.database import ProjectsData, Project, TasksData, Task

# Initialize Database connection
projects_data = ProjectsData()
tasks_data = TasksData()

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/projects_menu.ui')
class ProjectsMenu(Gtk.Box):
    __gtype_name__ = "ProjectsMenu"
    menu = Gtk.Template.Child()
    menu_button = Gtk.Template.Child()
    no_results = Gtk.Template.Child()
    projects_list = Gtk.Template.Child()
    archive_status: bool = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        # add actions
        self.connect("map", lambda *args: self.install_actions())

        self.fetch()

    # Actions
    def install_actions(self):
        actions = self.props.root.actions
        actions["search"].connect("activate", lambda *args: self.menu_button.popup())
        actions["update_project"].connect("activate", lambda *args: self.refresh_projects())
        actions["open_project"].connect("activate", lambda *args: self.open_project())

    @Gtk.Template.Callback()
    def new_project(self, sender):
        name = "New Project"
        project_id = projects_data.add(name)
        project = Project(id=project_id, name=name, archive=False)
        self.projects_list.append(SearchItem("project", project.name, project=project))
        self.props.root.project = project
        self.activate_action("win.open_project", GLib.Variant.new_tuple(
            GLib.Variant("b", True),
            GLib.Variant("i", -1)
        ))

    def refresh_projects(self):
        self.menu_button.set_label(self.props.root.project.name)
        self.clear()
        self.fetch()

    def open_project(self):
        self.menu_button.set_label(self.props.root.project.name)
        self.menu.popdown()

    @Gtk.Template.Callback()
    def on_search_entry_change(self, sender):
        self.clear()

        result = [[], []]
        text = sender.get_text().lower()

        if text:
            result[0] = projects_data.search(text, archive=self.archive_status)
            result[1] = tasks_data.search(text)
        else:
            self.fetch()
            self.no_results.set_visible(False)
            self.projects_list.set_visible(True)
            return

        if result == [[], []]:
            self.no_results.set_visible(True)
            self.projects_list.set_visible(False)
        else:
            self.no_results.set_visible(False)
            self.projects_list.set_visible(True)

            for project in result[0]:
                self.projects_list.append(
                    SearchItem("project", project.name, project=project))
            for task in result[1]:
                self.projects_list.append(
                    SearchItem("task", task.name, task=task))

    @Gtk.Template.Callback()
    def toggle_archive(self, sender):
        sender.set_has_frame(not sender.get_has_frame())
        self.archive_status = not self.archive_status

        self.refresh_projects()

    # UI
    def clear(self):
        while True:
            row = self.projects_list.get_first_child()
            if row:
                self.projects_list.remove(row)
            else:
                break

    def fetch(self):
        for project in projects_data.all(archive=self.archive_status):
            self.projects_list.append(SearchItem("project", project.name, project))


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/search_item.ui')
class SearchItem(Gtk.Button):
    __gtype_name__ = "SearchItem"
    name = Gtk.Template.Child()
    _type: str
    project: Project
    task: Task

    def __init__(self, _type, name, project=None, task=None):
        super().__init__()
        self._type = _type
        self.project = project
        self.task = task
        self.name.set_label(name)

    @Gtk.Template.Callback()
    def on_click(self, sender):
        if self._type == "project":
            self.props.root.project = self.project
            self.activate_action("win.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", -1)
            ))
        elif self._type == "task":
            self.props.root.project = projects_data.get(self.task.project)
            self.activate_action("win.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", self.task.id)
            ))
