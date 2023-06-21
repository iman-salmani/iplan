use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Task;

pub fn create_task(name: &str, project_id: i64, section_id: i64, parent: i64) -> Result<Task> {
    let position = new_position(section_id);
    let conn = get_connection();
    conn.execute(
        "INSERT INTO tasks(name, project, section, position, parent) VALUES (?1,?2,?3,?4,?5)",
        (name, project_id, section_id, position, parent),
    )?;
    Ok(Task::new(&[
        ("id", &conn.last_insert_rowid()),
        ("name", &name),
        ("project", &project_id),
        ("section", &section_id),
        ("position", &position),
        ("parent", &parent),
    ]))
}

pub fn read_tasks(
    project_id: Option<i64>,
    section_id: Option<i64>,
    done_tasks: Option<bool>,
    parent_id: Option<i64>,
    time_range: Option<(i64, i64)>,
) -> Result<Vec<Task>> {
    let filters = &mut vec![];
    if let Some(project_id) = project_id {
        filters.push(format!("project = {project_id}"));
    }
    if let Some(section_id) = section_id {
        filters.push(format!("section = {section_id}"));
    }
    if let Some(done_tasks) = done_tasks {
        filters.push(format!("done = {done_tasks}"));
    }
    if let Some(parent_id) = parent_id {
        filters.push(format!("parent = {parent_id}"));
    }
    if let Some((start, end)) = time_range {
        filters.push(format!("date >= {start} AND date < {end}"));
    }
    filters.push("suspended = 0".to_string());
    let filters_str = &mut String::new();
    for filter in filters {
        let prefix = if filters_str.is_empty() {
            "WHERE"
        } else {
            "AND"
        };
        filters_str.push_str(&format!("{prefix} {filter} "));
    }
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!(
        "SELECT * FROM tasks {filters_str} ORDER BY position DESC"
    ))?;
    let mut rows = stmt.query([])?;
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

pub fn update_task(task: &Task) -> Result<()> {
    let conn = get_connection();
    let old_task = read_task(task.id())?;
    let position_stmt = &mut String::new();

    if task.position() != old_task.position() {
        position_stmt.push_str(&format!("position = {},", task.position()));
        if task.parent() != old_task.parent() {
            if old_task.parent() == 0 {
                // Decrease tasks position in previous section
                conn.execute(
                    "UPDATE tasks SET position = position - 1
                    WHERE position > ?1 AND section = ?2",
                    (old_task.position(), old_task.section()),
                )?;
            } else {
                // Decrease subtasks position in previous parent
                conn.execute(
                    "UPDATE tasks SET position = position - 1
                    WHERE position > ?1 AND parent = ?2",
                    (old_task.position(), old_task.parent()),
                )?;
            }
        } else if task.section() != old_task.section() {
            // Decrease tasks position in previous section
            conn.execute(
                "UPDATE tasks SET position = position - 1
                WHERE position > ?1 AND section = ?2",
                (old_task.position(), old_task.section()),
            )?;

            // Increase tasks position in target section
            // Notify: Position not checked for value more than needed
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND section = ?2",
                (task.position(), task.section()),
            )?;
        } else if task.position() > old_task.position() {
            conn.execute(
                "UPDATE tasks SET position = position - 1
                WHERE position > ?1 AND position <= ?2 AND section = ?3",
                (old_task.position(), task.position(), task.section()),
            )?;
        } else if task.position() < old_task.position() {
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND position < ?2 AND section = ?3",
                (task.position(), old_task.position(), task.section()),
            )?;
        }
    }

    conn.execute(
        &format!(
            "UPDATE tasks SET
            name = ?2, done = ?3, project = ?4, section = ?5,
            {position_stmt} suspended = ?6, parent = ?7, description = ?8, date = ?9 WHERE id = ?1"
        ),
        (
            task.id(),
            task.name(),
            task.done(),
            task.project(),
            task.section(),
            task.suspended(),
            task.parent(),
            task.description(),
            task.date(),
        ),
    )?;
    Ok(())
}

pub fn delete_task(task_id: i64, section_id: i64, position: i32) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM tasks WHERE id = ?", (task_id,))?;
    conn.execute("DELETE FROM tasks WHERE parent = ?", (task_id,))?;
    conn.execute("DELETE FROM records WHERE task = ?", (task_id,))?;
    conn.execute("DELETE FROM reminders WHERE task = ?", (task_id,))?;
    // Decrease upper tasks position
    conn.execute(
        "UPDATE tasks SET position = position - 1 WHERE position > ?1 AND section = ?2",
        (position, section_id),
    )?;
    Ok(())
}

pub fn find_tasks(text: &str, done: bool) -> Result<Vec<Task>> {
    let filters = if done { "" } else { "AND done = false" };
    // Replace % and _ with \% and \_ because they have meaning
    // FIXME: do this without copy string
    let text = text.replace('%', r"\%").replace('_', r"\_");
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

pub fn new_position(section_id: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT position FROM tasks WHERE section = ? ORDER BY position DESC")
        .expect("Failed to find new task position");
    let first_row = stmt.query_row([section_id], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => first_row + 1,
        Err(_) => 0,
    }
}

pub fn new_subtask_position(parent: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT position FROM tasks WHERE parent = ? ORDER BY position DESC")
        .expect("Failed to find new task position");
    let first_row = stmt.query_row([parent], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => first_row + 1,
        Err(_) => 0,
    }
}
