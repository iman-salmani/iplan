from gi.repository import Gtk, Adw, GLib, Gdk
import os

from iplan.db.models.project import Project
from iplan.db.operations.project import create_project, read_projects, read_project
from iplan.db.operations.list import create_list
from iplan.views.sidebar.sidebar_project import SidebarProject


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar_projects.ui")
class SidebarProjects(Gtk.Box):
    __gtype_name__ = "SidebarProjects"
    projects_box: Gtk.Box = Gtk.Template.Child()
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
    def on_row_activated(self, list_box, row):
        window: Adw.Window = self.props.root
        window.props.application.project = row.project
        self.activate_action("app.open_project", GLib.Variant("i", -1))

        if window.get_size(Gtk.Orientation.HORIZONTAL) < 720:
            window.flap.set_reveal_flap(False)

    @Gtk.Template.Callback()
    def on_new_button_clicked(self, *args) -> None:
        name = "New Project"
        project = create_project(name)
        create_list("Tasks", project._id)
        self.projects_box.append(SidebarProject(project))
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
            row = self.projects_box.get_first_child()
            if row:
                self.projects_box.remove(row)
            else:
                break

    def fetch(self) -> None:
        selected_project: Project = self.props.root.props.application.project
        selected_project_row = None

        for project in read_projects(self.archive_button.get_active()):
            project_ui = SidebarProject(project)
            if project._id == selected_project._id:
                selected_project_row = project_ui
            self.projects_box.append(project_ui)

        if not selected_project_row:    # because archived
            project = read_project(selected_project._id)
            selected_project_row = SidebarProject(project)
            self.projects_box.append(selected_project_row)

        self.projects_box.select_row(selected_project_row)

