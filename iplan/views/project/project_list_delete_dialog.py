import gi
from gi.repository import Gtk, Adw, GLib

from iplan.db.operations.list import delete_list

@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_list_delete_dialog.ui")
class ProjectListDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectListDeleteDialog"
    project_list = None

    def __init__(self, project_list):
        super().__init__()
        self.project_list = project_list
        self.set_heading(
            f'Delete "{self.project_list._list.name}" Project?'
        )

    @Gtk.Template.Callback()
    def on_responsed(self, dialog, response):
        if response == "delete":
            delete_list(self.project_list._list._id)
            lists_box = self.project_list.get_parent()
            project_lists = lists_box.get_root().project_lists
            lists_box.remove(self.project_list)
            if not lists_box.get_first_child():
                lists_box.append(project_lists.placeholder)

