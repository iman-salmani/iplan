use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::List;

pub fn create_list(name: &str, project_id: i64) -> Result<List> {
    let index = new_index(project_id);
    let conn = get_connection();
    conn.execute(
        "INSERT INTO lists(name, project, i) VALUES (?1, ?2, ?3)",
        (name, project_id, index),
    )?;
    Ok(List::new(
        conn.last_insert_rowid(),
        String::from(name),
        project_id,
        index,
    ))
}

pub fn read_lists(project_id: i64) -> Result<Vec<List>> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM lists WHERE project = ? ORDER BY i ASC")?;
    let mut rows = stmt.query([project_id])?;
    let mut lists = Vec::new();
    while let Some(row) = rows.next()? {
        lists.push(List::try_from(row)?)
    }
    Ok(lists)
}

pub fn read_list(list_id: i64) -> Result<List> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM lists WHERE id = ?")?;
    stmt.query_row([list_id], |row| List::try_from(row))
}

pub fn update_list(list: &List) -> Result<()> {
    let conn = get_connection();
    let old_list = read_list(list.id())?;
    let index_stmt = &mut String::new();

    if list.index() != old_list.index() {
        index_stmt.push_str(&format!(", i = {}", list.index()));
        if list.index() > old_list.index() {
            conn.execute(
                "UPDATE lists SET i = i - 1
                WHERE i > ?1 AND i <= ?2",
                (old_list.index(), list.index()),
            )?;
        } else if list.index() < old_list.index() {
            conn.execute(
                "UPDATE lists SET i = i + 1
                WHERE i < ?1 AND i >= ?2",
                (old_list.index(), list.index()),
            )?;
        }
    }

    conn.execute(
        &format!(
            "UPDATE lists SET
            name = ?2, project = ?3, i = ?4 {index_stmt} WHERE id = ?1"
        ),
        (list.id(), list.name(), list.project(), list.index()),
    )?;
    Ok(())
}

pub fn delete_list(list_id: i64) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM lists WHERE id = ?", (list_id,))?;
    conn.execute("DELETE FROM tasks WHERE list = ?", (list_id,))?;
    Ok(())
}

fn new_index(project_id: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT i FROM lists WHERE project = ? ORDER BY i DESC")
        .expect("Failed to find new index");
    let first_row = stmt.query_row([project_id], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => return first_row + 1,
        Err(_) => return 0,
    };
}
