import gi
from gi.repository import Gtk, Adw

from iplan.db.operations.project import update_project
from iplan.views.project.project_delete_dialog import ProjectDeleteDialog

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_edit_window.ui")
class ProjectEditWindow(Adw.Window):
    __gtype_name__ = "ProjectEditWindow"
    name_entry_row: Adw.EntryRow = Gtk.Template.Child()
    archive_switch: Gtk.Switch = Gtk.Template.Child()

    def __init__(self, application):
        super().__init__(application=application)
        project = application.project
        self.name_entry_row.set_text(project.name)
        self.archive_switch.set_active(project.archive)

    @Gtk.Template.Callback()
    def name_entry_row_apply_cb(self, *args):
        self.get_application().project.name = self.name_entry_row.get_text()
        update_project(self.get_application().project)
        self.get_transient_for().activate_action("project.update")

    @Gtk.Template.Callback()
    def archive_switch_state_set_cb(self, sender: Gtk.Switch, state: bool):
        self.get_application().project.archive = state
        update_project(self.get_application().project)
        self.get_transient_for().activate_action("project.update")

    @Gtk.Template.Callback()
    def delete_button_clicked_cb(self, *args):
        dialog = ProjectDeleteDialog(self.get_application())
        dialog.set_transient_for(self)
        dialog.present()

