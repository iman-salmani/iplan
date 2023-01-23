from gi.repository import Gtk, Adw, GLib, Gdk
import os

from iplan.db.models.project import Project
from iplan.db.operations.project import create_project, read_projects
from iplan.db.operations.list import create_list
from iplan.views.sidebar.sidebar_project import SidebarProject


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar_projects.ui")
class SidebarProjects(Gtk.Box):
    __gtype_name__ = "SidebarProjects"
    projects_list: Gtk.Box = Gtk.Template.Child()
    archive_button: Gtk.ToggleButton = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.connect("map", self.on_mapped)

    # Actions
    def on_mapped(self, *args) -> None:
        "insert projects and Install actions after widget shown"
        self.disconnect_by_func(self.on_mapped)
        self.fetch()

        actions = self.props.root.props.application.actions
        actions["update_project"].connect("activate", self.refresh)
        # TODO: update only changed project
        actions["open_project"].connect("activate", self.refresh)
        # TODO: raise style for selected project instead of get projects again from database

    @Gtk.Template.Callback()
    def on_new_button_clicked(self, *args) -> None:
        name = "New Project"
        project = create_project(name)
        create_list("Tasks", project._id)
        self.projects_list.append(SidebarProject(project))
        self.props.root.props.application.project = project
        self.activate_action("app.open_project", GLib.Variant("i", -1))

    @Gtk.Template.Callback()
    def refresh(self, *args) -> None:
        # TODO: get only archived from database
        # instead of all projects when archive button is active
        self.clear()
        self.fetch()

    # UI
    def clear(self) -> None:
        while True:
            row = self.projects_list.get_first_child()
            if row:
                self.projects_list.remove(row)
            else:
                break

    def fetch(self) -> None:
        selected_project: Project = self.props.root.props.application.project
        for project in read_projects(self.archive_button.get_active()):
            project_ui = SidebarProject(project)
            if project._id == selected_project._id:
                project_ui.remove_css_class("flat")
                project_ui.add_css_class("raised")
            self.projects_list.append(project_ui)

