use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Task;

pub fn create_task(task: Task) -> Result<Task> {
    let conn = get_connection();
    conn.execute(
        "INSERT INTO tasks(name, project, section, position, parent, description, date) VALUES (?1,?2,?3,?4,?5,?6,?7)",
        (task.name(), task.project(), task.section(), task.position(), task.parent(), task.description(), task.date()),
    )?;
    task.set_id(conn.last_insert_rowid());
    Ok(task)
}

pub fn read_tasks(
    project_id: Option<i64>,
    section_id: Option<i64>,
    done_tasks: Option<bool>,
    parent_id: Option<i64>,
    time_range: Option<(i64, i64)>,
    suspended: bool,
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
    if !suspended {
        filters.push("suspended = false".to_string());
    }
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
        "SELECT * FROM tasks {filters_str} ORDER BY position"
    ))?;
    let mut rows = stmt.query([])?;
    let mut tasks = Vec::new();
    while let Some(row) = rows.next()? {
        tasks.push(Task::try_from(row)?)
    }
    Ok(tasks)
}

pub fn read_subtasks_summary(task_id: i64) -> Result<Vec<(String, bool)>> {
    let conn = get_connection();
    let mut stmt =
        conn.prepare("SELECT name, done FROM tasks WHERE parent = ?1 ORDER BY position DESC")?;
    let mut rows = stmt.query([task_id])?;
    let mut subtasks = Vec::new();
    while let Some(row) = rows.next()? {
        subtasks.push((row.get::<usize, String>(0)?, row.get::<usize, bool>(1)?));
    }
    Ok(subtasks)
}

pub fn task_tree(task_id: i64, has_date: bool) -> Result<Vec<i64>> {
    let conn = get_connection();
    let filter = if has_date { "WHERE date != 0" } else { "" };
    let mut stmt = conn.prepare(&format!(
        "WITH RECURSIVE task_tree(id, parent, date) AS (
	        SELECT id, parent, date FROM tasks WHERE id=?1
	        UNION ALL
	        SELECT tasks.id, tasks.parent, tasks.date
		        FROM tasks
		        JOIN task_tree ON tasks.parent=task_tree.id
        )
        SELECT id FROM task_tree {filter}",
    ))?;
    let mut rows = stmt.query([task_id])?;
    let mut subtasks = Vec::new();
    while let Some(row) = rows.next()? {
        subtasks.push(row.get::<usize, i64>(0)?);
    }

    Ok(subtasks)
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

    let task_position = task.position();
    let old_task_position = old_task.position();
    if task_position != old_task_position {
        position_stmt.push_str(&format!("position = {},", task_position));
        let old_task_parent = old_task.parent();
        if task.parent() != old_task_parent {
            if old_task_parent == 0 {
                // Decrease tasks position in previous section
                conn.execute(
                    "UPDATE tasks SET position = position - 1
                    WHERE position > ?1 AND section = ?2",
                    (old_task_position, old_task.section()),
                )?;
            } else {
                // Decrease subtasks position in previous parent
                conn.execute(
                    "UPDATE tasks SET position = position - 1
                    WHERE position > ?1 AND parent = ?2",
                    (old_task_position, old_task_parent),
                )?;
            }
        } else if task.section() != old_task.section() {
            // Decrease tasks position in previous section
            // Prevent from running, for old tasks without section
            if old_task.section() != 0 {
                conn.execute(
                    "UPDATE tasks SET position = position - 1
                    WHERE position > ?1 AND section = ?2",
                    (old_task_position, old_task.section()),
                )?;
            }

            // Increase tasks position in target section
            // Notify: Position not checked for value more than needed
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND section = ?2",
                (task_position, task.section()),
            )?;
        } else if task_position > old_task_position {
            conn.execute(
                "UPDATE tasks SET position = position - 1
                WHERE position > ?1 AND position <= ?2 AND section = ?3",
                (old_task_position, task_position, task.section()),
            )?;
        } else if task_position < old_task_position {
            conn.execute(
                "UPDATE tasks SET position = position + 1
                WHERE position >= ?1 AND position < ?2 AND section = ?3",
                (task_position, old_task_position, task.section()),
            )?;
        }
    }

    let task_project = task.project();
    if task_project != old_task.project() {
        conn.execute(
            "UPDATE tasks SET project = ?1 WHERE parent = ?2",
            (task_project, task.id()),
        )?;
    }

    let task_suspended = task.suspended();
    if task_suspended != old_task.suspended() {
        set_subtasks_suspended(&conn, task.id(), task_suspended)?;
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

fn set_subtasks_suspended(
    conn: &rusqlite::Connection,
    task_id: i64,
    suspended: bool,
) -> Result<()> {
    conn.execute(
        "UPDATE tasks SET suspended = ?1 WHERE parent = ?2",
        (suspended, task_id),
    )?;
    let mut stmt = conn.prepare("SELECT id FROM tasks WHERE parent = ?1")?;
    let mut rows = stmt.query((task_id,))?;
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        set_subtasks_suspended(conn, id, suspended)?;
    }
    Ok(())
}

pub fn delete_task(task: &Task) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    let task_id = task.id();
    conn.execute("DELETE FROM tasks WHERE id = ?", (task_id,))?;
    conn.execute("DELETE FROM records WHERE task = ?", (task_id,))?;
    conn.execute("DELETE FROM reminders WHERE task = ?", (task_id,))?;

    let subtasks = read_tasks(None, None, None, Some(task_id), None, false).unwrap();
    for subtask in subtasks {
        delete_task(&subtask).unwrap();
    }

    // Decrease upper tasks position
    if task.parent() == 0 {
        conn.execute(
            "UPDATE tasks SET position = position - 1 WHERE position > ?1 AND section = ?2",
            (task.position(), task.section()),
        )?;
    } else {
        conn.execute(
            "UPDATE tasks SET position = position - 1 WHERE position > ?1 AND parent = ?2",
            (task.position(), task.parent()),
        )?;
    }

    Ok(())
}

pub fn find_tasks(text: &str, done: bool) -> Result<Vec<Task>> {
    let mut filters = "AND suspended = false".to_string();
    if !done {
        filters.push_str(" AND done = false");
    };
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

pub fn new_task_position(section_id: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT position FROM tasks WHERE section = ? ORDER BY position DESC")
        .expect("Failed to find new task position");
    // FIXME: Do this inside the SQL query?
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
        .expect("Failed to find new subtask position");
    let first_row = stmt.query_row([parent], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => first_row + 1,
        Err(_) => 0,
    }
}

pub fn task_duration(task_id: i64) -> Result<i64> {
    let conn = get_connection();
    let mut stmt = conn.prepare(
        "WITH RECURSIVE task_tree(id, parent) AS (
	        SELECT id, parent FROM tasks WHERE id=?1
	        UNION ALL
	        SELECT tasks.id, tasks.parent
		        FROM tasks
		        JOIN task_tree ON tasks.parent=task_tree.id
        )
        SELECT coalesce(sum(duration), 0) FROM records JOIN task_tree ON records.task=task_tree.id;",
    )?;
    stmt.query_row([task_id], |row| row.get::<_, i64>(0))
}
