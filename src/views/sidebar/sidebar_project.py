from gi.repository import Gtk, GLib, Gdk, Adw

from iplan.db.models.project import Project
from iplan.db.models.task import Task
from iplan.db.operations.list import read_lists
from iplan.db.operations.task import update_task, find_new_task_position
from iplan.views.project.project_list_task import ProjectListTask


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar_project.ui")
class SidebarProject(Gtk.ListBoxRow):
    __gtype_name__ = "SidebarProject"
    project: Project = None
    name_label: Gtk.Label = Gtk.Template.Child()

    def __init__(self, project):
        super().__init__()
        self.project = project

        self.name_label.set_label(self.project.name)

        if project.archive:
            self.name_label.add_css_class("dim-label")

        drop_task_target = Gtk.DropTarget.new(ProjectListTask, Gdk.DragAction.MOVE)
        drop_task_target.set_preload(True)
        drop_task_target.connect("drop", self.drop_task_target_drop_cb)
        drop_task_target.connect("motion", self.drop_task_target_motion_cb)
        self.add_controller(drop_task_target)

    @Gtk.Template.Callback()
    def drag_prepare_cb(
        self, drag_source: Gtk.DragSource, x: float, y: float
    ) -> Gdk.ContentProvider:
        return Gdk.ContentProvider.new_for_value(self)

    @Gtk.Template.Callback()
    def drag_begin_cb(self, drag_source: Gtk.DragSource, drag: Gdk.Drag):
        self.get_parent().select_row(self)
        drag_icon = Gtk.DragIcon.get_for_drag(drag)
        drag_icon.props.child = Gtk.Label()
        drag.set_hotspot(0, 0)

    @Gtk.Template.Callback()
    def drag_cancel_cb(
        self, drag_source: Gtk.DragSource, drag: Gdk.Drag, reason
    ) -> bool:
        self.get_parent().get_parent().select_active_project()
        return False

    def drop_task_target_drop_cb(
        self, target: Gtk.DropTarget, source_task_row: ProjectListTask, x, y
    ) -> bool:
        source_task_row.task.project = self.project._id
        source_task_row.task._list = list(read_lists(self.project._id))[0]._id
        source_task_row.task.position = find_new_task_position(
            source_task_row.task._list
        )
        source_task_row.get_parent().remove(source_task_row)
        update_task(source_task_row.task, move_position=True)
        self.get_parent().get_parent().select_active_project()
        return True

    def drop_task_target_motion_cb(self, target: Gtk.DropTarget, x, y):
        source_task_row: ProjectListTask = target.get_value()
        if source_task_row.task.project == self.project._id:
            return 0
        self.get_parent().select_row(self)
        return Gdk.DragAction.MOVE
