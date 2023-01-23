import gi
from gi.repository import Gtk, Adw

from iplan.db.operations.project import update_project
from iplan.views.project.project_delete_dialog import ProjectDeleteDialog

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_edit_window.ui")
class ProjectEditWindow(Adw.Window):
    __gtype_name__ = "ProjectEditWindow"
    name: Adw.EntryRow = Gtk.Template.Child()
    archive: Gtk.Switch = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.connect("map", self.on_mapped)

    def on_mapped(self, *args):
        self.disconnect_by_func(self.on_mapped)
        project = self.get_application().project
        self.name.set_text(project.name)
        self.archive.set_active(project.archive)

    @Gtk.Template.Callback()
    def on_name_applied(self, *args):
        self.get_application().project.name = self.name.get_text()
        update_project(self.get_application().project)
        self.activate_action("app.update_project")

    @Gtk.Template.Callback()
    def on_archive_state_seted(self, sender: Gtk.Switch, state: bool):
        self.get_application().project.archive = state
        update_project(self.get_application().project)
        self.activate_action("app.update_project")

    @Gtk.Template.Callback()
    def on_delete_clicked(self, *args):
        dialog = ProjectDeleteDialog(self.get_application())
        dialog.set_transient_for(self.get_root())
        dialog.present()

