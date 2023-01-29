import gi
from gi.repository import Gtk, Adw, GLib

from iplan.db.operations.project import delete_project, read_projects

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_delete_dialog.ui")
class ProjectDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectDeleteDialog"
    application = None

    def __init__(self, application):
        super().__init__()
        self.application = application
        self.set_heading(
            f'Delete "{application.project.name}" Project?'
        )

    @Gtk.Template.Callback()
    def response_cb(self, dialog, response):
        if response == "delete":
            delete_project(self.application.project)
            window = self.get_toplevels()[0]
            window.activate_action(
                "project.delete",
                GLib.Variant("i", self.application.project.index)
            )

            #TODO: move this to window?
            projects = read_projects()
            if not projects:
               self.application.project = read_projects(archive=True)[0]
            self.application.project = list(projects)[0]
            window.activate_action("project.open")

            self.get_transient_for().close()

