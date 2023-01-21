import gi
from gi.repository import Gtk, Adw, GLib
from time import sleep
from datetime import datetime

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/page/project_header.ui")
class ProjectHeader(Gtk.Box):
    __gtype_name__ = "ProjectHeader"
    project_name: Gtk.Label = Gtk.Template.Child()
    project_duration_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button_content: Adw.ButtonContent = Gtk.Template.Child()
    project_duration_records: Gtk.Box = Gtk.Template.Child()
    project_options_popover: Gtk.Popover = Gtk.Template.Child()
    new_task_button: Gtk.Button = Gtk.Template.Child()
    show_completed_tasks_switch: Gtk.Switch = Gtk.Template.Child()
    archive_project_switch: Gtk.Switch = Gtk.Template.Child()
    delete_project_button: Gtk.Button = Gtk.Template.Child()

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.archive_project_switch.connect("notify::state", self.toggle_archive_project)
        self.connect("map", lambda *args: self.install_actions())

    # Actions
    def install_actions(self):
        actions = self.props.root.props.application.actions

        actions["open_project"].connect(
            "activate",
            lambda *args: self.open_project(args[1][0])
        )

        actions["refresh_project_duration"].connect(
            "activate",
            lambda *args: self.refresh_project_duration()
        )

        self.show_completed_tasks_switch.connect(
            "state-set",
            lambda *args: actions["toggle_completed_tasks"].change_state(
                GLib.Variant('b', args[1])
            )
        )

    def refresh_project_duration(self):
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

    def open_project(self, new):
        self.project_name.set_text(self.props.root.props.application.project.name)

        self.refresh_project_duration()

        self.archive_project_switch.handler_block_by_func(self.toggle_archive_project)
        self.archive_project_switch.set_state(self.props.root.props.application.project.archive)
        self.archive_project_switch.handler_unblock_by_func(self.toggle_archive_project)

        self.show_completed_tasks_switch.set_state(False)

    def toggle_archive_project(self, sender, param):
        state = sender.get_state()
        self.props.root.props.application.project.archive = state
        projects_data.update(self.props.root.props.application.project)
        self.activate_action("app.update_project")

        if state:
            self.props.root.props.application.project = projects_data.first()
            self.activate_action("app.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", -1)
            ))
            self.project_options_popover.popdown()

    @Gtk.Template.Callback()
    def open_project_delete_dialog(self, sender):
        self.project_options_popover.popdown()
        dialog = ProjectDeleteDialog()
        dialog.set_heading(f'Delete "{self.props.root.props.application.project.name}" Project?')
        dialog.set_transient_for(self.get_root())
        dialog.connect("response", self.on_project_delete_dialog_response)
        dialog.present()

    def on_project_delete_dialog_response(self, dialog, response):
        if response == "delete":
            projects_data.delete(self.props.root.props.application.project.id)
            self.activate_action("app.update_project")
            self.props.root.props.application.project = projects_data.first()
            self.activate_action("app.open_project", GLib.Variant.new_tuple(
                GLib.Variant("b", False),
                GLib.Variant("i", -1)
            ))

    # UI Functions
    def clear(self, box):
        while True:
            row = box.get_first_child()
            if row:
                box.remove(row)
            else:
                break


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project_delete_dialog.ui")
class ProjectDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectDeleteDialog"

    def __init__(self):
        super().__init__()

