from gi.repository import Gtk, GLib, Gdk, Adw

from iplan.db.models.project import Project
from iplan.db.models.task import Task
from iplan.db.operations.list import read_lists
from iplan.db.operations.task import update_task, find_new_task_position
from iplan.views.project.project_list_task import ProjectListTask


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/sidebar/sidebar_project.ui")
class SidebarProject(Gtk.Button):
    __gtype_name__ = "SidebarProject"
    project: Project = None
    content: Gtk.Label = Gtk.Template.Child()

    def __init__(self, project):
        super().__init__()
        self.project = project

        if project.archive:
            self.content.set_label(f"<s>{self.project.name}</s>")
        else:
            self.content.set_label(self.project.name)

        drop_target = Gtk.DropTarget.new(ProjectListTask, Gdk.DragAction.MOVE)
        drop_target.set_preload(True)
        drop_target.connect("drop", self.on_dropped)
        drop_target.connect("motion", self.on_motioned)
        self.add_controller(drop_target)

    @Gtk.Template.Callback()
    def open_project(self, *args):
        window: Adw.Window = self.props.root
        window.props.application.project = self.project
        self.activate_action("app.open_project", GLib.Variant("i", -1))

        if window.get_size(Gtk.Orientation.HORIZONTAL) < 720:
            window.flap.set_reveal_flap(False)

    def on_dropped(
            self,
            target: Gtk.DropTarget,
            source_widget: ProjectListTask,
            x: float, y: float) -> bool:
        source_widget.task.project = self.project._id
        source_widget.task._list = list(read_lists(self.project._id))[0]._id
        # TODO: open project and to prefered list
        # do this after change list drop system for create space for new task intead of replace
        # show completed tasks if source task done
        source_widget.task.position = find_new_task_position(source_widget.task._list)
        update_task(source_widget.task)
        # TODO: remove just dropped task
        self.activate_action("app.refresh_tasks")

    def on_motioned(self, target: Gtk.DropTarget, x, y):
        source_widget: ProjectListTask = target.get_value()
        if source_widget.task.project == self.project._id:
            return 0
        return Gdk.DragAction.MOVE

