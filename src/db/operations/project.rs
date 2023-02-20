use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Project;

pub fn create_project(name: &str) -> Result<Project> {
    let index = new_index();
    let conn = get_connection();
    conn.execute(
        "INSERT INTO projects(name, i) VALUES (?1,?2)",
        (name, index),
    )?;
    Ok(Project::new(
        conn.last_insert_rowid(),
        String::from(name),
        false,
        index,
    ))
}

pub fn read_projects(archive: bool) -> Result<Vec<Project>> {
    let filters = if !archive { "WHERE archive = false" } else { "" };
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!("SELECT * FROM projects {filters} ORDER BY i ASC"))?;
    let mut rows = stmt.query([])?;
    let mut projects = Vec::new();
    while let Some(row) = rows.next()? {
        projects.push(Project::try_from(row)?)
    }
    Ok(projects)
}

pub fn read_project(project_id: i64) -> Result<Project> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM projects WHERE id = ?")?;
    stmt.query_row([project_id], |row| Project::try_from(row))
}

pub fn update_project(project: &Project) -> Result<()> {
    let conn = get_connection();
    let old_project = read_project(project.id())?;
    let index_stmt = &mut String::new();

    if project.index() != old_project.index() {
        index_stmt.push_str(&format!(", i = {}", project.index()));
        if project.index() > old_project.index() {
            conn.execute(
                "UPDATE projects SET i = i - 1
                WHERE i > ?1 AND i <= ?2",
                (old_project.index(), project.index()),
            )?;
        } else if project.index() < old_project.index() {
            conn.execute(
                "UPDATE projects SET i = i + 1
                WHERE i < ?1 AND i >= ?2",
                (old_project.index(), project.index()),
            )?;
        }
    }

    conn.execute(
        &format!(
            "UPDATE projects SET
            name = ?1, archive = ?2 {index_stmt} WHERE id = ?3"
        ),
        (project.name(), project.archive(), project.id()),
    )?;
    Ok(())
}

pub fn delete_project(project_id: i64, index: i32) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM projects WHERE id = ?", (project_id,))?;
    conn.execute("DELETE FROM lists WHERE project = ?", (project_id,))?;
    conn.execute("DELETE FROM tasks WHERE project = ?", (project_id,))?;
    // Decrease upper projects index
    conn.execute("UPDATE projects SET i = i - 1 WHERE i > ?1", (index,))?;
    Ok(())
}

pub fn find_projects(text: &str, archive: bool) -> Result<Vec<Project>> {
    let filters = if archive { "" } else { "AND archive = false" };
    // Replace % and _ with \% and \_ because they have meaning
    // TODO: do this whitout copy string
    let text = text.replace("%", r"\%").replace("_", r"\_");
    let conn = get_connection();
    let mut stmt = conn.prepare(&format!(
        "SELECT * FROM projects WHERE name LIKE ? ESCAPE '\\' {filters}"
    ))?;
    let mut rows = stmt.query([format!("%{text}%")])?;
    let mut projects = Vec::new();
    while let Some(row) = rows.next()? {
        projects.push(Project::try_from(row)?)
    }
    Ok(projects)
}

fn new_index() -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT i FROM projects ORDER BY i DESC")
        .expect("Failed to find new index");
    let first_row = stmt.query_row([], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => return first_row + 1,
        Err(_) => return 0,
    };
}
