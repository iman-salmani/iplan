from gi.repository import Gtk, Adw, Gio, GLib
import os

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/utility_pane.ui")
class UtilityPane(Gtk.Box):
    __gtype_name__ = "UtilityPane"
    projects_list: Gtk.Box = Gtk.Template.Child()
    archive_button: Gtk.ToggleButton = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.connect("map", self.on_map)

    # Actions
    def on_map(self, *args):
        "insert projects and Install actions after widget mapped"

        self.refresh()

        actions = self.props.root.props.application.actions
        actions["update_project"].connect("activate", self.refresh)
        actions["open_project"].connect("activate", self.refresh)

    @Gtk.Template.Callback()
    def new_project(self, sender):
        name = "New Project"
        project_id = projects_data.add(name)
        project = Project(id=project_id, name=name, archive=False)
        self.projects_list.append(UtilityPaneProjectsItem(project))
        self.props.root.project = project
        self.activate_action("app.open_project", GLib.Variant.new_tuple(
            GLib.Variant("b", True),
            GLib.Variant("i", -1)
        ))

    @Gtk.Template.Callback()
    def refresh(self, *args):
        self.clear()
        self.fetch()

    # UI
    def clear(self):
        while True:
            row = self.projects_list.get_first_child()
            if row:
                self.projects_list.remove(row)
            else:
                break

    def fetch(self):
        selected_project = self.props.root.project
        for project in projects_data.all(archive=self.archive_button.get_active()):
            project_ui = UtilityPaneProjectsItem(project)

            if project.id == selected_project.id:
                project_ui.remove_css_class("flat")
                project_ui.add_css_class("suggested-action")

            self.projects_list.append(project_ui)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/utility_pane_projects_item.ui")
class UtilityPaneProjectsItem(Gtk.Button):
    __gtype_name__ = "UtilityPaneProjectsItem"
    project: Project = None
    content: Gtk.Label = Gtk.Template.Child()

    def __init__(self, project):
        super().__init__()
        self.project = project

        if project.archive:
            self.content.set_label(f"<s>{self.project.name}</s>")
        else:
            self.content.set_label(self.project.name)


    @Gtk.Template.Callback()
    def open_project(self, sender):
        self.props.root.project = self.project
        self.activate_action("app.open_project", GLib.Variant.new_tuple(
            GLib.Variant("b", False),
            GLib.Variant("i", -1)
        ))

