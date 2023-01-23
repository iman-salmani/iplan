import gi
from gi.repository import Gtk, Adw

from iplan.views.project.project_edit_window import ProjectEditWindow


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project/project_header.ui")
class ProjectHeader(Gtk.Box):
    __gtype_name__ = "ProjectHeader"
    project_name: Gtk.Label = Gtk.Template.Child()
    project_duration_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button_content: Adw.ButtonContent = Gtk.Template.Child()
    project_duration_records: Gtk.Box = Gtk.Template.Child()

    def __init__(self):
        super().__init__()
        self.connect("map", self.on_mapped)

    # Actions
    def on_mapped(self, *args):
        self.disconnect_by_func(self.on_mapped)
        actions = self.props.root.props.application.actions

        actions["open_project"].connect("activate", self.open_project)

        # TODO: think about mix update project and refresh duration
        actions["update_project"].connect(
            "activate",
            lambda *args: self.project_name.set_text(
                self.props.root.props.application.project.name
            )
        )

        actions["refresh_project_duration"].connect(
            "activate",
            self.refresh_project_duration
        )

        actions["edit_project"].connect(
            "activate",
            self.present_edit_project_window
        )

    def open_project(self, *args):
        self.project_name.set_text(self.props.root.props.application.project.name)
        self.refresh_project_duration()

    def present_edit_project_window(self, *args):
        window = ProjectEditWindow()
        window.set_application(self.props.root.get_application())
        window.set_transient_for(self.get_root())
        window.present()

    def refresh_project_duration(self, *args):
        duration = self.props.root.props.application.project.get_duration()
        if duration:
            self.project_duration_button_content.set_label(
                self.props.root.props.application.project.duration_to_text(duration)
            )
        else:
            self.project_duration_button_content.set_label("")

        table = self.props.root.props.application.project.get_duration_table()
        self.clear(self.project_duration_records)
        dates = list(table.keys())
        dates.sort()
        dates.reverse()
        for date in dates:
            box = Gtk.Box()
            box.set_margin_top(9)
            box.set_margin_bottom(9)
            box.set_margin_start(9)
            box.set_margin_end(9)
            self.project_duration_records.append(box)

            date_label = Gtk.Label()
            date_label.set_text(date.strftime("%d %b"))
            date_label.set_margin_end(18)
            date_label.set_hexpand(True)
            date_label.set_halign(Gtk.Align.START)
            box.append(date_label)

            duration_label = Gtk.Label()
            duration_label.set_text(self.props.root.props.application.project.duration_to_text(table[date]))
            box.append(duration_label)

            if date != dates[-1]:
                self.project_duration_records.append(Gtk.Separator())

    # UI Functions
    def clear(self, box):
        while True:
            row = box.get_first_child()
            if row:
                box.remove(row)
            else:
                break

