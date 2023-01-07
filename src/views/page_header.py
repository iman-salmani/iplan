import gi
from gi.repository import Gtk, Adw, GLib
from time import sleep
from datetime import datetime

from iplan.database.database import ProjectsData, Project

# Initialize Database connection
projects_data = ProjectsData()


class PageHeader(Gtk.Box):
    __gtype_name__ = "PageHeader"
    project = None

    def __init__(self, signal_controller):
        super().__init__()
        self.signal_controller = signal_controller
        self.set_margin_top(12)
        self.set_margin_start(15)
        self.set_margin_end(24)
        self.set_orientation(Gtk.Orientation.HORIZONTAL)
        self.add_css_class("toolbar")

        # add Handlers
        self.signal_controller.add_handler("project-open", self.opened_project)
        self.signal_controller.add_handler("todo-duration-update", self.update_duration)

        # Project name
        self.project_name_button = Gtk.Button()
        self.project_name_button.add_css_class("flat")
        self.project_name_button.connect("clicked", self.click_edit_project_name)
        self.project_name_button.set_halign(Gtk.Align.START)
        self.append(self.project_name_button)

        self.project_name_label = Gtk.Label()
        self.project_name_label.add_css_class("heading")
        self.project_name_button.set_child(self.project_name_label)

        self.project_name_entry = Gtk.Entry()
        self.project_name_entry.add_css_class("heading")
        self.project_name_entry.set_margin_start(1)
        self.project_name_entry.set_hexpand(True)
        self.project_name_entry.set_visible(False)
        self.project_name_entry.connect("activate", self.change_project_name)
        self.append(self.project_name_entry)

        buffer = Gtk.EntryBuffer()
        self.project_name_entry.set_buffer(buffer)

        # Project durations
        self.durations_button = Gtk.MenuButton()
        self.durations_button.add_css_class("flat")
        self.append(self.durations_button)

        self.durations_button_content = Adw.ButtonContent()
        self.durations_button_content.set_icon_name("preferences-system-time-symbolic")
        self.durations_button.set_child(self.durations_button_content)

        durations_popover = Gtk.Popover()
        self.durations_button.set_popover(durations_popover)
        self.durations_stat = Gtk.Box()
        self.durations_stat.set_orientation(Gtk.Orientation.VERTICAL)
        durations_popover.set_child(self.durations_stat)

        # Separator
        self.separator = Gtk.Separator()
        self.separator.set_hexpand(True)
        self.separator.add_css_class("spacer")
        self.append(self.separator)

        # Edit project name
        self.edit_project_name_button = Gtk.Button.new_from_icon_name(
            "document-edit-symbolic"
        )
        self.edit_project_name_button.set_has_frame(False)
        self.edit_project_name_button.connect("clicked", self.click_edit_project_name)
        self.append(self.edit_project_name_button)

        self.change_project_name_button = Gtk.Button.new_from_icon_name(
            "document-edit-symbolic"
        )
        self.change_project_name_button.set_visible(False)
        self.change_project_name_button.add_css_class("suggested-action")
        self.change_project_name_button.add_css_class("circular")
        self.change_project_name_button.connect("clicked", self.change_project_name)
        self.append(self.change_project_name_button)

        new_button = Gtk.Button.new_from_icon_name("list-add-symbolic")
        new_button.set_has_frame(False)
        new_button.connect(
            "clicked", lambda sender: self.signal_controller.emit_signal("todo-new")
        )
        self.append(new_button)

        self.append(self.create_options_ui())

    # Communicate with database
    def delete(self, sender):
        projects_data.delete(self.project.id)
        self.signal_controller.emit_signal("project-update")
        self.signal_controller.emit_signal("project-open", projects_data.first())
        self.options_popover.popdown()

    # Signal Handlers
    def change_project_name(self, sender):
        self.project.name = self.project_name_entry.get_buffer().get_text()
        projects_data.update(self.project)
        self.signal_controller.emit_signal("project-update")
        self.project_name_label.set_text(self.project.name)
        self.change_status("show")

    def update_duration(self):
        duration = self.project.get_duration()
        if duration:
            self.durations_button_content.set_label(
                self.project.duration_to_text(duration)
            )
        else:
            self.durations_button_content.set_label("")

        table = self.project.get_duration_table()
        self.clear(self.durations_stat)
        dates = list(table.keys())
        dates.sort()
        dates.reverse()
        for date in dates:
            box = Gtk.Box()
            box.set_margin_top(9)
            box.set_margin_bottom(9)
            box.set_margin_start(9)
            box.set_margin_end(9)
            self.durations_stat.append(box)

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
                self.durations_stat.append(Gtk.Separator())

    def opened_project(self, project: Project):
        self.project = project

        self.project_name_entry.get_buffer().set_text(project.name, -1)
        self.project_name_label.set_text(project.name)
        self.change_status("show")

        self.update_duration()

        self.signal_controller.block_signal("project-open")
        self.archive_switch.set_state(project.archive)
        self.signal_controller.unblock_signal("project-open")

    def click_edit_project_name(self, sender):
        self.change_status("edit")

        self.project_name_entry.grab_focus_without_selecting()

    def toggle_archive_project(self, sender, state):
        self.project.archive = state
        projects_data.update(self.project)
        self.signal_controller.emit_signal("project-update")

        if state:
            self.signal_controller.emit_signal("project-open", projects_data.first())
            self.options_popover.popdown()

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
            self.edit_project_name_button.set_visible(False)
            self.durations_button.set_visible(False)
            self.separator.set_visible(False)

            self.change_project_name_button.set_visible(True)
            self.project_name_entry.set_visible(True)
        else:
            self.project_name_entry.set_visible(False)
            self.change_project_name_button.set_visible(False)

            self.project_name_button.set_visible(True)
            self.durations_button.set_visible(True)
            self.separator.set_visible(True)
            self.edit_project_name_button.set_visible(True)

    def create_options_ui(self):
        options_button = Gtk.MenuButton()
        options_button.set_icon_name("view-more-symbolic")
        options_button.set_has_frame(False)

        self.options_popover = Gtk.Popover()
        self.options_popover.set_halign(Gtk.Align.END)
        self.options_popover.set_valign(Gtk.Align.START)
        self.options_popover.set_has_arrow(False)
        self.options_popover.set_offset(0, 6)
        options_button.set_popover(self.options_popover)

        options = Gtk.Box()
        options.set_orientation(Gtk.Orientation.VERTICAL)
        self.options_popover.set_child(options)

        # Switch show completed tasks
        show_completed_tasks_switch = Gtk.Switch()
        show_completed_tasks_switch.set_margin_start(24)
        show_completed_tasks_switch.connect(
            "state-set",
            lambda sender, state: self.signal_controller.emit_signal(
                "todo-toggle_completed", show_completed_tasks_switch.get_active()
            ),
        )
        options.append(
            self.create_option_ui("Completed tasks", show_completed_tasks_switch)
        )

        # Separator
        separator = Gtk.Separator()
        separator.set_margin_top(6)
        separator.set_margin_bottom(6)
        options.append(separator)

        # Archive project
        self.archive_switch = Gtk.Switch()
        self.archive_switch.set_margin_start(24)
        self.archive_switch.connect("state-set", self.toggle_archive_project)
        options.append(self.create_option_ui("Archive project", self.archive_switch))

        # Delete project
        delete = Gtk.Button.new_from_icon_name("app-remove-symbolic")
        delete.add_css_class("flat")
        delete.connect("clicked", self.delete)
        options.append(self.create_option_ui("Delete project", delete))

        return options_button

    def create_option_ui(self, name, option):
        row = Gtk.Box()
        row.set_margin_top(6)
        row.set_margin_bottom(6)
        row.set_margin_start(6)
        row.set_margin_end(6)

        label = Gtk.Label.new(name)
        label.set_hexpand(True)
        label.set_halign(Gtk.Align.START)
        row.append(label)
        option.set_valign(Gtk.Align.CENTER)
        row.append(option)

        return row

