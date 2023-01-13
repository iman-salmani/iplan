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
    project = None
    project_name_button: Gtk.Button = Gtk.Template.Child()
    project_name_button_label: Gtk.Label = Gtk.Template.Child()
    project_name_entry: Gtk.Entry = Gtk.Template.Child()
    project_name_edit_button: Gtk.Button = Gtk.Template.Child()
    project_name_apply_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button: Gtk.Button = Gtk.Template.Child()
    project_duration_button_content: Adw.ButtonContent = Gtk.Template.Child()
    project_duration_records: Gtk.Box = Gtk.Template.Child()
    project_options_popover: Gtk.Popover = Gtk.Template.Child()
    new_todo_button: Gtk.Button = Gtk.Template.Child()
    separator: Gtk.Separator = Gtk.Template.Child()
    show_completed_tasks_switch: Gtk.Switch = Gtk.Template.Child()
    archive_project_switch: Gtk.Switch = Gtk.Template.Child()
    delete_project_button: Gtk.Button = Gtk.Template.Child()

    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)

        self.project_name_button.connect("clicked", self.click_edit_project_name)
        self.project_name_entry.connect("activate", self.change_project_name)
        self.project_name_edit_button.connect("clicked", self.click_edit_project_name)
        self.project_name_apply_button.connect("clicked", self.change_project_name)
        self.archive_project_switch.connect("notify::state", self.toggle_archive_project)
        self.delete_project_button.connect("clicked", self.delete)

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

    def change_project_name(self, sender):
        self.project.name = self.project_name_entry.get_buffer().get_text()
        projects_data.update(self.project)
        self.props.root.actions["update_project"].activate()
        self.project_name_button_label.set_text(self.project.name)
        self.change_status("show")

    def refresh_project_duration(self):
        duration = self.project.get_duration()
        if duration:
            self.project_duration_button_content.set_label(
                self.project.duration_to_text(duration)
            )
        else:
            self.project_duration_button_content.set_label("")

        table = self.project.get_duration_table()
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
            duration_label.set_text(self.project.duration_to_text(table[date]))
            box.append(duration_label)

            if date != dates[-1]:
                self.project_duration_records.append(Gtk.Separator())

    def open_project(self, project_id: int):
        self.project = projects_data.get(project_id)
        self.project_name_entry.get_buffer().set_text(self.project.name, -1)
        self.project_name_button_label.set_text(self.project.name)

        self.refresh_project_duration()

        self.archive_project_switch.handler_block_by_func(self.toggle_archive_project)
        self.archive_project_switch.set_state(self.project.archive)
        self.archive_project_switch.handler_unblock_by_func(self.toggle_archive_project)

        self.show_completed_tasks_switch.set_state(False)

    def click_edit_project_name(self, sender):
        self.change_status("edit")

        self.project_name_entry.grab_focus_without_selecting()

    def toggle_archive_project(self, sender, param):
        state = sender.get_state()
        self.project.archive = state
        projects_data.update(self.project)
        self.activate_action("win.update_project")

        if state:
            self.activate_action(
                "win.open_project",
                GLib.Variant('i', projects_data.first().id)
            )
            self.project_options_popover.popdown()

    def delete(self, sender):
        projects_data.delete(self.project.id)
        actions = self.props.root.actions
        actions["update_project"].activate()
        self.activate_action(
            "win.open_project",
            GLib.Variant('i', projects_data.first().id)
        )
        self.project_options_popover.popdown()

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

