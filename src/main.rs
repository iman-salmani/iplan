/* main.rs
 *
 * Copyright 2023 Iman Salmani
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

mod application;
mod config;
mod db;
mod views;

use self::application::IPlanApplication;

use config::{APPLICATION_ID, GETTEXT_PACKAGE, LOCALEDIR, PKGDATADIR};
use gettextrs::{bind_textdomain_codeset, bindtextdomain, textdomain};
use gtk::gio;
use gtk::prelude::*;

fn main() {
    // Set up gettext translations
    bindtextdomain(GETTEXT_PACKAGE, LOCALEDIR).expect("Unable to bind the text domain");
    bind_textdomain_codeset(GETTEXT_PACKAGE, "UTF-8")
        .expect("Unable to set the text domain encoding");
    textdomain(GETTEXT_PACKAGE).expect("Unable to switch to the text domain");

    // Load resources
    let resources = gio::Resource::load(PKGDATADIR.to_owned() + "/iplan.gresource")
        .expect("Could not load resources");
    gio::resources_register(&resources);

    // Check database
    db::check_database().expect("Database check failed");

    // Test create list
    // let list = db::operations::create_list("INSERT';", 1)
    //     .expect("Failed to create list");
    // println!("{list}");

    // Test read lists
    // let lists = db::operations::read_lists(1)
    //     .expect("Failed to read lists");
    // if let Some(list) = lists.get(0) {
    //     println!("{}", list);
    // }

    // Test read list
    // let list = db::operations::read_list(8)
    //     .expect("Failed to read list");
    // println!("{list}");

    // Test update list
    // db::operations::update_list(db::models::List{
    //         id: 8,
    //         name: "New Name".to_string(),
    //         project: 1,
    //         index: 1
    //     }).expect("Failed to update list");

    // Test delete list
    // db::operations::delete_list(9)
    //     .expect("Failed to delete list");

    // Test create task
    // let task = db::operations::create_task("Test Task;", 1, 1).expect("Failed to create task");
    // println!("{task:?}");

    // Test read tasks
    // let tasks = db::operations::read_tasks(1, None, None).expect("Failed to read tasks");
    // println!("{}", tasks.len());

    // Test read task
    // let task = db::operations::read_task(1)
    //     .expect("Failed to read list");
    // println!("{task}");

    // Test update task
    // db::operations::update_task(db::models::Task{
    //     id: 2,
    //     name: String::from("task 3"),
    //     done: false,
    //     project: 1,
    //     list: 2,
    //     duration: String::new(),
    //     position: 0,
    //     suspended: false,
    // }).expect("Failed to update list");

    // Test delete task
    // db::operations::delete_task(3, 1, 1)
    //     .expect("Failed to delete task");

    // Test find tasks
    // let tasks = db::operations::find_tasks(r"task 1_\%", false)
    //     .expect("Failed to find tasks");
    // println!("{}", tasks.len());

    // Test create project
    // let project = db::operations::create_project("Test Project 2")
    //     .expect("Failed to create project");
    // println!("{project}");

    // Test read projects
    // let projects = db::operations::read_projects(false)
    //     .expect("Failed to read projects");
    // println!("{}", projects.len());

    // Test read project
    // let project = db::operations::read_project(1)
    //     .expect("Failed to read project");
    // println!("{project}");

    // Test update project
    // db::operations::update_project(db::models::Project{
    //     id: 2,
    //     name: String::from("Test Project"),
    //     archive: false,
    //     index: 2,
    // }).expect("Failed to update project");

    // Test delete project
    // db::operations::delete_project(1, 0)
    //     .expect("Failed to delete project");

    // Test find projects
    // let projects = db::operations::find_projects(r"2", false)
    //     .expect("Failed to find projects");
    // println!("{}", projects.len());

    // Create a new GtkApplication. The application manages our main loop,
    // application windows, integration with the window manager/compositor, and
    // desktop features such as file opening and single-instance applications.

    let app = IPlanApplication::new(APPLICATION_ID, &gio::ApplicationFlags::empty());

    // Run the application. This function will block until the application
    // exits. Upon return, we have our exit code to return to the shell. (This
    // is the code you see when you do `echo $?` after running a command in a
    // terminal.
    std::process::exit(app.run());
}
