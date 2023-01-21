import gi
from gi.repository import Gtk, Adw, GLib

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/page/edit_project_window.ui")
class EditProjectWindow(Adw.Window):
    __gtype_name__ = "EditProjectWindow"
    name: Adw.EntryRow = Gtk.Template.Child()
    archive: Gtk.Switch = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.connect("map", self.on_mapped)

    def on_mapped(self, *args):
        project = self.get_application().project
        self.name.set_text(project.name)
        self.archive.set_active(project.archive)

    @Gtk.Template.Callback()
    def on_name_applied(self, *args):
        self.get_application().project.name = self.name.get_text()
        projects_data.update(self.get_application().project)
        self.activate_action("app.update_project")

    @Gtk.Template.Callback()
    def on_archive_state_seted(self, sender: Gtk.Switch, state: bool):
        self.get_application().project.archive = state
        projects_data.update(self.get_application().project)
        self.activate_action("app.update_project")

    @Gtk.Template.Callback()
    def on_delete_clicked(self, *args):
        dialog = ProjectDeleteDialog(self.get_application())
        dialog.set_transient_for(self.get_root())
        dialog.present()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project_delete_dialog.ui")
class ProjectDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectDeleteDialog"
    app = None

    def __init__(self, app):
        super().__init__()
        self.app = app
        self.set_heading(
            f'Delete "{self.app.project.name}" Project?'
        )

    @Gtk.Template.Callback()
    def on_responsed(self, dialog, response):
        if response == "delete":
            projects_data.delete(self.app.project.id)
            self.activate_action("app.update_project")
            self.app.project = projects_data.first()
            self.app.activate_action("open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", -1)
            ))
            self.get_transient_for().close()
