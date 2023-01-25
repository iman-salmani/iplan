from gi.repository import Gtk, Adw, GLib, Gdk
import os

from iplan.db.models.project import Project
from iplan.db.operations.project import create_project, read_projects, read_project, update_project
from iplan.db.operations.list import create_list
from iplan.views.sidebar.sidebar_project import SidebarProject


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar_projects.ui")
class SidebarProjects(Gtk.Box):
    __gtype_name__ = "SidebarProjects"
    projects_box: Gtk.Box = Gtk.Template.Child()
    archive_button: Gtk.ToggleButton = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.projects_box.set_filter_func(self._filter)
        self.projects_box.set_sort_func(self.sort)
        self.connect("map", self.on_mapped)
        drop_target = Gtk.DropTarget.new(SidebarProject, Gdk.DragAction.MOVE)
        drop_target.set_preload(True)
        drop_target.connect("drop", self.on_dropped)
        drop_target.connect("motion", self.on_motioned)
        self.projects_box.add_controller(drop_target)

    # Actions
    def on_mapped(self, *args) -> None:
        "insert projects and Install actions after widget shown"
        self.disconnect_by_func(self.on_mapped)
        self.fetch()

        actions = self.props.root.props.application.actions
        actions["update_project"].connect("activate", self.refresh)
        # TODO: update only changed project
        #actions["open_project"].connect("activate", self.on_project_opened)
        self.props.root.props.application.create_action(
            "open-searched",
            callback=self.select_active_project
            )
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

    def refresh(self, *args) -> None:
        # TODO: get only archived from database
        # instead of all projects when archive button is active
        self.clear()
        self.fetch()

    def select_active_project(self, *args):
        project = self.props.root.props.application.project
        target_row = None
        for row in self.projects_box.observe_children():
            if row.project._id == project._id:
                target_row = row
                break
        self.projects_box.select_row(target_row)
        if project.archive and not self.archive_button.get_active():
            archive_button.toggled()

    @Gtk.Template.Callback()
    def on_archive_button_toggled(self, *args) -> None:
        self.projects_box.invalidate_filter()

    # UI
    def on_dropped(self, target: Gtk.DropTarget, source_row, x, y):
        # source_row moved by motion signal so it should drop on itself
        project_in_db = read_project(source_row.project._id)
        if project_in_db != source_row.project:
            update_project(source_row.project, move_position=True)
        self.select_active_project()
        return True

    def on_motioned(self, target: Gtk.DropTarget, x, y):
        source_row = target.get_value()
        target_row = self.projects_box.get_row_at_y(y)

        # None check
        if not source_row or not target_row:
            return 0

        # Move shadow_row
        if source_row != target_row:
            # index is reverse of position
            shadow_i = source_row.get_index()
            target_i = target_row.get_index()
            target_p = target_row.project.position
            if shadow_i == target_i - 1:
                source_row.project.position -= 1
                target_row.project.position +=1
            elif shadow_i < target_i:
                for i in range(shadow_i+1, target_i+1):
                    row = self.projects_box.get_row_at_index(i)
                    row.project.position += 1
                source_row.project.position = target_p
            elif shadow_i == target_i + 1:
                source_row.project.position += 1
                target_row.project.position -=1
            elif shadow_i > target_i:
                for i in range(target_i, shadow_i):
                    row = self.projects_box.get_row_at_index(i)
                    row.project.position -= 1
                source_row.project.position = target_p

            self.projects_box.invalidate_sort()

        return Gdk.DragAction.MOVE

    def _filter(self, row) -> bool:
        if self.archive_button.get_active():
            return True
        if self.projects_box.get_selected_row() == row:
            return True
        return not row.project.archive

    def sort(self, row1, row2) -> int:
        return row2.project.position - row1.project.position

    def clear(self) -> None:
        while True:
            row = self.projects_box.get_first_child()
            if row:
                self.projects_box.remove(row)
            else:
                break

    def fetch(self) -> None:
        selected_project: Project = self.props.root.props.application.project
        target_row = None

        for project in read_projects(archive=True):
            project_ui = SidebarProject(project)
            if project._id == selected_project._id:
                target_row = project_ui
            self.projects_box.append(project_ui)

        self.projects_box.select_row(target_row)

