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
    content: Gtk.Label = Gtk.Template.Child()

    def __init__(self, project):
        super().__init__()
        self.project = project

        self.content.set_label(self.project.name)

        if project.archive:
            self.content.add_css_class("dim-label")

        drop_target = Gtk.DropTarget.new(ProjectListTask, Gdk.DragAction.MOVE)
        drop_target.set_preload(True)
        drop_target.connect("drop", self.on_dropped)
        drop_target.connect("motion", self.on_motioned)
        self.add_controller(drop_target)

    def on_dropped(
            self,
            target: Gtk.DropTarget,
            source_task_row: ProjectListTask,
            x: float, y: float) -> bool:
        source_task_row.task.project = self.project._id
        source_task_row.task._list = list(read_lists(self.project._id))[0]._id
        # TODO: open project and to prefered list
        # do this after change list drop system for create space for new task intead of replace
        # show completed tasks if source task done
        source_task_row.task.position = find_new_task_position(source_task_row.task._list)
        source_task_row.get_parent().remove(source_task_row)
        update_task(source_task_row.task, move_position=True)
        return True

    def on_motioned(self, target: Gtk.DropTarget, x, y):
        source_task_row: ProjectListTask = target.get_value()
        if source_task_row.task.project == self.project._id:
            return 0
        return Gdk.DragAction.MOVE

