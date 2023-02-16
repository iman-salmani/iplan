use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Record;

pub fn create_record(start: i64, task_id: i64) -> Result<Record> {
    let conn = get_connection();
    conn.execute(
        "INSERT INTO records(start, task) VALUES (?1,?2)",
        (start, task_id),
    )?;
    Ok(Record::new(conn.last_insert_rowid(), start, 0, task_id))
}

pub fn read_records(task_id: i64, uncomplete: bool) -> Result<Vec<Record>> {
    let filters = if uncomplete { "AND duration = 0" } else { "" };
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!(
        "SELECT * FROM records WHERE task = ? {filters} ORDER BY start DESC"
    ))?;
    let mut rows = stmt.query([task_id])?;
    let mut records = Vec::new();
    while let Some(row) = rows.next()? {
        records.push(Record::try_from(row)?)
    }
    Ok(records)
}

pub fn read_record(record_id: i64) -> Result<Record> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM records WHERE id = ?")?;
    stmt.query_row([record_id], |row| Record::try_from(row))
}

pub fn update_record(record: &Record) -> Result<()> {
    let conn = get_connection();
    conn.execute(
        &format!(
            "UPDATE records SET
            start = ?1, duration = ?2, task = ?3 WHERE id = ?4"
        ),
        (
            record.start(),
            record.duration(),
            record.task(),
            record.id(),
        ),
    )?;
    Ok(())
}

pub fn delete_record(record_id: i64) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM records WHERE id = ?", (record_id,))?;
    Ok(())
}
