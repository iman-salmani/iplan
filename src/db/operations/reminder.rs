use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Reminder;

pub fn create_reminder(datetime: i64, task_id: i64, priority: u8) -> Result<Reminder> {
    let conn = get_connection();
    conn.execute(
        "INSERT INTO reminders(datetime, task, priority) VALUES (?1,?2,?3)",
        (datetime, task_id, priority),
    )?;
    Ok(Reminder::new(
        conn.last_insert_rowid(),
        datetime,
        false,
        task_id,
        priority,
    ))
}

pub fn read_reminders(task_id: i64) -> Result<Vec<Reminder>> {
    let conn = get_connection();
    let mut stmt =
        conn.prepare("SELECT * FROM reminders WHERE task = ? AND past = 0 ORDER BY datetime DESC")?;
    let mut rows = stmt.query([task_id])?;
    let mut reminders = Vec::new();
    while let Some(row) = rows.next()? {
        reminders.push(Reminder::try_from(row)?)
    }
    Ok(reminders)
}

pub fn read_reminder(reminder_id: i64) -> Result<Reminder> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM reminders WHERE id = ?")?;
    stmt.query_row([reminder_id], |row| Reminder::try_from(row))
}

pub fn update_reminder(reminder: &Reminder) -> Result<()> {
    let conn = get_connection();
    conn.execute(
        "UPDATE reminders SET datetime = ?2, past = ?3, task = ?4, priority = ?5 WHERE id = ?1",
        (
            reminder.id(),
            reminder.datetime(),
            reminder.past(),
            reminder.task(),
            reminder.priority(),
        ),
    )?;
    Ok(())
}

pub fn delete_reminder(reminder_id: i64) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM reminders WHERE id = ?", (reminder_id,))?;
    Ok(())
}
