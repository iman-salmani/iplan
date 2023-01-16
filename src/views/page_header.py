import gi
from gi.repository import Gtk, Adw, GLib
from time import sleep
from datetime import datetime

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/page_header.ui")
class PageHeader(Gtk.Box):
    __gtype_name__ = "PageHeader"
    project_name_button: Gtk.Button = Gtk.Template.Child()
    project_name_button_label: Gtk.Label = Gtk.Template.Child()
    project_name_entry: Gtk.Entry = Gtk.Template.Child()
    project_name_edit_button: Gtk.Button = Gtk.Template.Child()
    project_name_apply_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button_content: Adw.ButtonContent = Gtk.Template.Child()
    project_duration_records: Gtk.Box = Gtk.Template.Child()
    project_options_popover: Gtk.Popover = Gtk.Template.Child()
    new_task_button: Gtk.Button = Gtk.Template.Child()
    separator: Gtk.Separator = Gtk.Template.Child()
    show_completed_tasks_switch: Gtk.Switch = Gtk.Template.Child()
    archive_project_switch: Gtk.Switch = Gtk.Template.Child()
    delete_project_button: Gtk.Button = Gtk.Template.Child()

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.archive_project_switch.connect("notify::state", self.toggle_archive_project)
        self.connect("map", lambda *args: self.install_actions())

    # Actions
    def install_actions(self):
        actions = self.props.root.actions

        actions["open_project"].connect(
            "activate",
            lambda *args: self.open_project(args[1])
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

    @Gtk.Template.Callback()
    def change_project_name(self, sender):
        self.props.root.project.name = self.project_name_entry.get_buffer().get_text()
        projects_data.update(self.props.root.project)
        self.activate_action("win.update_project")
        self.project_name_button_label.set_text(self.props.root.project.name)
        self.change_status("show")

    def refresh_project_duration(self):
        duration = self.props.root.project.get_duration()
        if duration:
            self.project_duration_button_content.set_label(
                self.props.root.project.duration_to_text(duration)
            )
        else:
            self.project_duration_button_content.set_label("")

        table = self.props.root.project.get_duration_table()
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
            duration_label.set_text(self.props.root.project.duration_to_text(table[date]))
            box.append(duration_label)

            if date != dates[-1]:
                self.project_duration_records.append(Gtk.Separator())

    def open_project(self, new):
        self.project_name_entry.get_buffer().set_text(self.props.root.project.name, -1)
        self.project_name_button_label.set_text(self.props.root.project.name)

        if new:
            self.change_status("edit")
            self.project_name_entry.grab_focus()

        self.refresh_project_duration()

        self.archive_project_switch.handler_block_by_func(self.toggle_archive_project)
        self.archive_project_switch.set_state(self.props.root.project.archive)
        self.archive_project_switch.handler_unblock_by_func(self.toggle_archive_project)

        self.show_completed_tasks_switch.set_state(False)

    @Gtk.Template.Callback()
    def click_edit_project_name(self, sender):
        self.change_status("edit")

        self.project_name_entry.grab_focus_without_selecting()

    def toggle_archive_project(self, sender, param):
        state = sender.get_state()
        self.props.root.project.archive = state
        projects_data.update(self.props.root.project)
        self.activate_action("win.update_project")

        if state:
            self.props.root.project = projects_data.first()
            self.activate_action("win.open_project", GLib.Variant("b", False))
            self.project_options_popover.popdown()

    @Gtk.Template.Callback()
    def open_project_delete_dialog(self, sender):
        self.project_options_popover.popdown()
        window = self.get_root()
        dialog = ProjectDeleteDialog()
        dialog.set_transient_for(window)
        dialog.set_modal(True)
        dialog.set_destroy_with_parent(True)
        dialog.connect("response", self.delete_project)
        dialog.present()

    def delete_project(self, dialog, response):
        if response == "delete":
            projects_data.delete(self.props.root.project.id)
            self.activate_action("win.update_project")
            self.props.root.project = projects_data.first()
            self.activate_action("win.open_project", GLib.Variant("b", False))

    # UI Functions
    def clear(self, box):
        while True:
            row = box.get_first_child()
            if row:
                box.remove(row)
            else:
                break

    def change_status(self, status):
        if status == "edit":
            self.project_name_button.set_visible(False)
            self.project_name_edit_button.set_visible(False)
            self.project_duration_button.set_visible(False)
            self.separator.set_visible(False)

            self.project_name_apply_button.set_visible(True)
            self.project_name_entry.set_visible(True)
        else:
            self.project_name_entry.set_visible(False)
            self.project_name_apply_button.set_visible(False)

            self.project_name_button.set_visible(True)
            self.project_duration_button.set_visible(True)
            self.separator.set_visible(True)
            self.project_name_edit_button.set_visible(True)


@Gtk.Template(resource_path="/ir/imansalmani/iplan/ui/project_delete_dialog.ui")
class ProjectDeleteDialog(Adw.MessageDialog):
    __gtype_name__ = "ProjectDeleteDialog"

