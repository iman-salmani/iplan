use std::env;
use std::path::PathBuf;
use rusqlite::{Connection, Result};
use gtk::glib;

pub fn get_connection() -> Result<Connection> {
    Connection::open(get_database_path())
}

pub fn check_database() -> Result<()> {
    // Create database if not exists

    let database_path = get_database_path();

    if !database_path.exists() {
        let conn = Connection::open(database_path)?;

        conn.execute(
            "CREATE TABLE projects (
                id	      INTEGER NOT NULL,
                name	  TEXT    NOT NULL,
                archive   INTEGER NOT NULL DEFAULT 0,
                i         INTEGER NOT NULL,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;
        conn.execute("INSERT INTO projects(name, i) VALUES ('Personal', 0)", ())?;

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
        conn.execute("INSERT INTO lists(name, project, i) VALUES ('Tasks', 1, 0)", ())?;

        conn.execute(
            "CREATE TABLE tasks (
                id	     INTEGER NOT NULL,
                name	     TEXT    NOT NULL,
                done	     INTEGER NOT NULL DEFAULT False,
                project    INTEGER NOT NULL,
                list       INTEGER NOT NULL,
                duration   TEXT    NOT NULL DEFAULT '',
                position   INTEGER NOT NULL,
                suspended  INTEGER NOT NULL DEFAULT False,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;
    }
    Ok(())
}

fn get_database_path() -> PathBuf {
    let dir_path = if env::var("INSIDE_GNOME_BUILDER").is_ok() {
        glib::user_cache_dir()
    } else {
        glib::user_data_dir()
    };
    dir_path.join("data.db")
}

