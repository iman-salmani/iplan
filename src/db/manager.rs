use gtk::glib;
use rusqlite::{Connection, Result};

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
                id	      INTEGER NOT NULL,
                name	  TEXT    NOT NULL,
                archive   INTEGER NOT NULL DEFAULT 0,
                i         INTEGER NOT NULL,
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
                id	     INTEGER NOT NULL,
                name	     TEXT    NOT NULL,
                done	     INTEGER NOT NULL DEFAULT 0,
                project    INTEGER NOT NULL,
                list       INTEGER NOT NULL,
                duration   TEXT    NOT NULL DEFAULT '',
                position   INTEGER NOT NULL,
                suspended  INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY(id AUTOINCREMENT)
            );",
            (),
        )?;
    }
    Ok(())
}
