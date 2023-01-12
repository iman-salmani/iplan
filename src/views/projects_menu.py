import gi
from gi.repository import Gtk, Adw, GLib, Gio

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()

@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/projects_menu.ui')
class ProjectsMenu(Gtk.Box):
    __gtype_name__ = "ProjectsMenu"
    menu = Gtk.Template.Child()
    menu_button = Gtk.Template.Child()
    search = Gtk.Template.Child()
    no_results = Gtk.Template.Child()
    projects_list = Gtk.Template.Child()
    archive = Gtk.Template.Child()
    archive_status: bool = False

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        # add actions
        self.search.connect("search-changed", self.search_changed)
        self.archive.connect("clicked", self.toggle_archive)
        self.connect("map", lambda *args: self.install_actions())

        self.fetch()

    # Actions
    def install_actions(self):
        actions = self.props.root.actions
        actions["update_project"].connect("activate", lambda *args: self.refresh_projects())
        actions["new_project"].connect("activate", lambda *args: self.new())
        actions["open_project"].connect("activate", lambda *args: self.open_project(args[1]))

    def new(self):
        name = "New Project"
        project_id = projects_data.add(name)
        project = Project(id=project_id, name=name, archive=False)
        self.projects_list.append(ProjectsMenuItem(project))
        self.activate_action("win.open_project", GLib.Variant('i', project_id))

    def refresh_projects(self):
        self.clear()
        self.fetch()

    def open_project(self, project_id):
        project = projects_data.get(project_id)
        self.menu_button.set_label(project.name)
        self.menu.popdown()

    def search_changed(self, sender):
        self.clear()

        text = sender.get_text()
        if text:
            projects = projects_data.search(text, archive=self.archive_status)
            if projects:
                for project in projects:
                    self.projects_list.append(ProjectsMenuItem(project))
                self.no_results.set_visible(False)
                self.projects_list.set_visible(True)
            else:
                self.no_results.set_visible(True)
                self.projects_list.set_visible(False)
        else:
            self.fetch()
            self.no_results.set_visible(False)
            self.projects_list.set_visible(True)

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
            self.projects_list.append(ProjectsMenuItem(project))


@Gtk.Template(resource_path='/ir/imansalmani/iplan/ui/projects_menu_item.ui')
class ProjectsMenuItem(Gtk.Button):
    __gtype_name__ = "ProjectsMenuItem"
    name = Gtk.Template.Child()

    def __init__(self, project: Project, **kwargs):
        super().__init__(**kwargs)

        self.set_action_target_value(GLib.Variant('i', project.id))
        self.name.set_label(project.name)

