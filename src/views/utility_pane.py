from gi.repository import Gtk, Adw, Gio, GLib
import os

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/utility_pane.ui")
class UtilityPane(Gtk.Box):
    __gtype_name__ = "UtilityPane"
    projects_list: Gtk.Box = Gtk.Template.Child()

    def __init__(self):
        super().__init__()

        for project in projects_data.all():
            project_ui = UtilityPaneProjectsItem(project)
            self.projects_list.append(project_ui)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/utility_pane_projects_item.ui")
class UtilityPaneProjectsItem(Gtk.Button):
    __gtype_name__ = "UtilityPaneProjectsItem"
    project: Project = None
    content: Gtk.Label = Gtk.Template.Child()

    def __init__(self, project):
        super().__init__()
        self.project = project

        self.content.set_label(self.project.name)

    @Gtk.Template.Callback()
    def open_project(self, sender):
        self.props.root.project = self.project
        self.activate_action("app.open_project", GLib.Variant.new_tuple(
            GLib.Variant("b", False),
            GLib.Variant("i", -1)
        ))

