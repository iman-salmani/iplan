use rusqlite::Result;

use crate::db::get_connection;

pub static MIGRATIONS: [fn() -> Result<()>; 8] = [to1, to2, to3, to4, to5, to6, to7, to8];

fn to1() -> Result<()> {
    // Create records from duration column in tasks table and drop it.
    // before:
    // task sql:
    // "id"	        INTEGER NOT NULL,
    // "name"	    TEXT NOT NULL,
    // "done"	    INTEGER NOT NULL DEFAULT 0,
    // "project"	INTEGER NOT NULL,
    // "list"	    INTEGER NOT NULL,
    // "duration"	TEXT NOT NULL DEFAULT '',
    // "position"	INTEGER NOT NULL,
    // "suspended"	INTEGER NOT NULL DEFAULT 0,
    // task duration example: 1671365268.58338,6224;1671378590.05254,4336;
    // after
    // record sql:
    // "id"         INTEGER NOT NULL,
    // "start"      INTEGER NOT NULL,
    // "duration"	INTEGER NOT NULL DEFAULT 0,
    // "task"   	INTEGER NOT NULL,
    let conn = get_connection();

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

    let mut stmt = conn.prepare("SELECT * FROM tasks")?;
    let mut rows = stmt.query(())?;
    while let Some(row) = rows.next()? {
        let mut duration_column = row.get::<usize, String>(5)?;
        if !duration_column.is_empty() {
            duration_column.pop();
            for raw_record in duration_column.split(';') {
                let start = &raw_record[0..raw_record.find('.').unwrap()];
                let duration_int = &raw_record[raw_record.find(',').unwrap() + 1..];
                conn.execute(
                    "INSERT INTO records(start, duration, task) VALUES (?1, ?2, ?3)",
                    (start, duration_int, row.get::<usize, i64>(0)?),
                )?;
            }
        }
    }

    conn.execute("ALTER TABLE tasks DROP COLUMN duration;", ())?;

    Ok(())
}

fn to2() -> Result<()> {
    // Add parent column to tasks table
    let conn = get_connection();
    conn.execute(
        "ALTER TABLE tasks ADD parent INTEGER NOT NULL DEFAULT 0;",
        (),
    )?;
    Ok(())
}

fn to3() -> Result<()> {
    // Add icon column to projects table
    let conn = get_connection();
    conn.execute(
        "ALTER TABLE projects ADD icon TEXT NOT NULL DEFAULT '';",
        (),
    )?;
    Ok(())
}

fn to4() -> Result<()> {
    // Add description column to tasks table
    let conn = get_connection();
    conn.execute(
        "ALTER TABLE tasks ADD description TEXT NOT NULL DEFAULT '';",
        (),
    )?;
    Ok(())
}

fn to5() -> Result<()> {
    // Add description column to projects table
    let conn = get_connection();
    conn.execute(
        "ALTER TABLE projects ADD description TEXT NOT NULL DEFAULT '';",
        (),
    )?;
    Ok(())
}

fn to6() -> Result<()> {
    // Add date column to tasks table
    let conn = get_connection();
    conn.execute("ALTER TABLE tasks ADD date INTEGER NOT NULL DEFAULT 0;", ())?;
    Ok(())
}

fn to7() -> Result<()> {
    // Create reminders table
    let conn = get_connection();
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
    Ok(())
}

fn to8() -> Result<()> {
    // Rename Lists table to Sections
    let conn = get_connection();
    conn.execute("ALTER TABLE lists RENAME TO sections;", ())?;
    conn.execute("ALTER TABLE tasks RENAME COLUMN list TO section;", ())?;
    Ok(())
}
