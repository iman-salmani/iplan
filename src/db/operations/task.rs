use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Task;

pub fn create_task(name: &str, project_id: i64, list_id: i64) -> Result<Task> {
    let position = new_position(list_id);
    let conn = get_connection();
    conn.execute(
        "INSERT INTO tasks(name, project, list, position) VALUES (?1,?2,?3,?4)",
        (name, project_id, list_id, position),
    )?;
    Ok(Task::new(
        conn.last_insert_rowid(),
        String::from(name),
        false,
        project_id,
        list_id,
        position,
        false,
    ))
}

pub fn read_tasks(
    project_id: i64,
    list_id: Option<i64>,
    done_tasks: Option<bool>,
) -> Result<Vec<Task>> {
    let filters = &mut String::new();
    if let Some(list_id) = list_id {
        filters.push_str(&format!(" AND list = {list_id}"));
    }
    if let Some(done_tasks) = done_tasks {
        filters.push_str(&format!(" AND done = {done_tasks}"));
    }
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!(
        "SELECT * FROM tasks WHERE project = ? {filters} ORDER BY position DESC"
    ))?;
    let mut rows = stmt.query([project_id])?;
    let mut tasks = Vec::new();
    while let Some(row) = rows.next()? {
        tasks.push(Task::try_from(row)?)
    }
    Ok(tasks)
}

pub fn read_task(task_id: i64) -> Result<Task> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM tasks WHERE id = ?")?;
    stmt.query_row([task_id], |row| Task::try_from(row))
}

pub fn update_task(task: Task) -> Result<()> {
    let conn = get_connection();
    let old_task = read_task(task.id())?;
    let position_stmt = &mut String::new();

    if task.position() != old_task.position() {
        position_stmt.push_str(&format!("position = {},", task.position()));
        if task.list() != old_task.list() {
            // Decrease tasks position in previous list
            conn.execute(
                "UPDATE tasks SET position = position - 1
                WHERE position > ?1 AND list = ?2",
                (old_task.position(), old_task.list()),
            )?;

            // Increase tasks position in target list
            // Notify: Position not checked for value more than needed
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND list = ?2",
                (task.position(), task.list()),
            )?;
        } else if task.position() > old_task.position() {
            conn.execute(
                "UPDATE tasks SET position = position - 1
                WHERE position > ?1 AND position <= ?2 AND list = ?3",
                (old_task.position(), task.position(), task.list()),
            )?;
        } else if task.position() < old_task.position() {
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND position < ?2 AND list = ?3",
                (task.position(), old_task.position(), task.list()),
            )?;
        }
    }

    conn.execute(
        &format!(
            "UPDATE tasks SET
            name = ?1, done = ?2, project = ?3, list = ?4,
            {position_stmt} suspended = ?5 WHERE id = ?6"
        ),
        (
            task.name(),
            task.done(),
            task.project(),
            task.list(),
            task.suspended(),
            task.id(),
        ),
    )?;
    Ok(())
}

pub fn delete_task(task_id: i64, list_id: i64, position: i32) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM tasks WHERE id = ?", (task_id,))?;
    // Decrease upper tasks position
    conn.execute(
        "UPDATE tasks SET position = position - 1 WHERE position > ?1 AND list = ?2",
        (position, list_id),
    )?;
    Ok(())
}

pub fn find_tasks(text: &str, done: bool) -> Result<Vec<Task>> {
    let filters = if done { "" } else { "AND done = false" };
    // Replace % and _ with \% and \_ because they have meaning
    // TODO: do this whitout copy string
    let text = text.replace("%", r"\%").replace("_", r"\_");
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!(
        "SELECT * FROM tasks WHERE name LIKE ? ESCAPE '\\' {filters}"
    ))?;
    let mut rows = stmt.query([format!("%{text}%")])?;
    let mut tasks = Vec::new();
    while let Some(row) = rows.next()? {
        tasks.push(Task::try_from(row)?)
    }
    Ok(tasks)
}

pub fn new_position(list_id: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT position FROM tasks WHERE list = ? ORDER BY position DESC")
        .expect("Failed to find new task position");
    let first_row = stmt.query_row([list_id], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => return first_row + 1,
        Err(_) => return 0,
    };
}
