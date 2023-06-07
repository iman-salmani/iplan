use gtk::glib;
use rusqlite::{Connection, Result};
use std::cmp::Ordering;

use crate::db::migrate::MIGRATIONS;

const DB_VERSION: u8 = 7;

pub fn get_connection() -> Connection {
    Connection::open(glib::user_data_dir().join("data.db")).expect("Failed connect to database")
}

pub fn check_database() -> Result<()> {
    // Create database if not exists

    let database_path = glib::user_data_dir().join("data.db");

    if !database_path.exists() {
        let conn = Connection::open(database_path)?;

        conn.execute(
            "CREATE TABLE projects (
                id	        INTEGER NOT NULL,
                name	    TEXT    NOT NULL,
                archive     INTEGER NOT NULL DEFAULT 0,
                i           INTEGER NOT NULL,
                icon        TEXT    NOT NULL DEFAULT '',
                description TEXT    NOT NULL DEFAULT '',
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;

        conn.execute(
            "CREATE TABLE lists (
                id        INTEGER NOT NULL,
                name      TEXT    NOT NULL,
                project   INTEGER NOT NULL,
                i         INTEGER NOT NULL,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;

        conn.execute(
            "CREATE TABLE tasks (
                id	        INTEGER NOT NULL,
                name	    TEXT    NOT NULL,
                done	    INTEGER NOT NULL DEFAULT 0,
                project     INTEGER NOT NULL,
                list        INTEGER NOT NULL,
                position    INTEGER NOT NULL,
                suspended   INTEGER NOT NULL DEFAULT 0,
                parent      INTEGER NOT NULL DEFAULT 0,
                description TEXT    NOT NULL DEFAULT '',
                date	    INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;

        conn.execute(
            "CREATE TABLE records (
                id	      INTEGER NOT NULL,
                start	  INTEGER NOT NULL,
                duration  INTEGER NOT NULL DEFAULT 0,
                task      INTEGER NOT NULL,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;

        conn.execute(
            "CREATE TABLE reminders (
                id	      INTEGER NOT NULL,
                datetime  INTEGER NOT NULL,
                past      INTEGER NOT NULL DEFAULT 0,
                task      INTEGER NOT NULL,
                priority  INTEGER NOT NULL DEFAULT 1,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;

        conn.execute(&format!("PRAGMA user_version={}", DB_VERSION), ())?;
    } else {
        let conn = get_connection();
        // conn.pragma(schema_name, pragma_name, pragma_value, f)
        let mut stmt = conn.prepare("PRAGMA user_version")?;
        let version = stmt.query_row([], |row| row.get::<usize, u8>(0)).unwrap();
        match DB_VERSION.cmp(&version) {
            Ordering::Greater => {
                for i in version..DB_VERSION {
                    MIGRATIONS[i as usize]().expect("Failed to migrate database");
                    conn.execute(&format!("PRAGMA user_version={}", DB_VERSION), ())?;
                }
            }
            Ordering::Less => {
                panic!(
                    "Database version is {}. please update application.",
                    version
                );
            }
            Ordering::Equal => {}
        }
    }
    Ok(())
}
