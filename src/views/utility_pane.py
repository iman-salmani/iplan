from gi.repository import Gtk, Adw, GLib, Gdk
import os

from iplan.database.database import ProjectsData, Project, TasksData, Task
from iplan.views.page_item import TaskRow

# Initialize Database connection
projects_data = ProjectsData()
tasks_data = TasksData()

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/utility_pane.ui")
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
        self.props.root.props.application.project = project
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
        selected_project = self.props.root.props.application.project
        for project in projects_data.all(archive=self.archive_button.get_active()):
            project_ui = UtilityPaneProjectsItem(project)

            if project.id == selected_project.id:
                project_ui.remove_css_class("flat")
                project_ui.add_css_class("raised")

            self.projects_list.append(project_ui)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/utility_pane_projects_item.ui")
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

        drop_target = Gtk.DropTarget.new(TaskRow, Gdk.DragAction.MOVE)
        drop_target.set_preload(True)
        drop_target.connect("drop", self.on_drop)
        drop_target.connect("motion", self.on_motion)
        self.add_controller(drop_target)

    @Gtk.Template.Callback()
    def open_project(self, sender):
        window = self.props.root
        window.props.application.project = self.project
        self.activate_action("app.open_project", GLib.Variant.new_tuple(
            GLib.Variant("b", False),
            GLib.Variant("i", -1)
        ))

        window_width = window.get_size(Gtk.Orientation.HORIZONTAL)
        if window_width < 720:
            window.flap.set_reveal_flap(False)

    def on_drop(
            self,
            target: Gtk.DropTarget,
            source_widget: TaskRow,
            x: float, y: float) -> bool:
        source_widget.task.project = self.project.id
        source_widget.task.position = tasks_data.new_position(self.project.id)
        tasks_data.update(source_widget.task)
        self.activate_action("app.refresh_tasks")

    def on_motion(self, target: Gtk.DropTarget, x, y):
        source_widget: TaskRow = target.get_value()
        if source_widget.task.project == self.project.id:
            return 0
        return Gdk.DragAction.MOVE

