use std::cmp::Ordering;

use rusqlite::Result;

use crate::db::get_connection;
use crate::db::models::Section;

pub fn create_section(name: &str, project_id: i64) -> Result<Section> {
    let index = new_index(project_id);
    let conn = get_connection();
    conn.execute(
        "INSERT INTO sections(name, project, i) VALUES (?1, ?2, ?3)",
        (name, project_id, index),
    )?;
    Ok(Section::new(
        conn.last_insert_rowid(),
        String::from(name),
        project_id,
        index,
    ))
}

pub fn read_sections(project_id: i64) -> Result<Vec<Section>> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM sections WHERE project = ? ORDER BY i ASC")?;
    let mut rows = stmt.query([project_id])?;
    let mut sections = Vec::new();
    while let Some(row) = rows.next()? {
        sections.push(Section::try_from(row)?)
    }
    Ok(sections)
}

pub fn read_section(section_id: i64) -> Result<Section> {
    let conn = get_connection();
    let mut stmt = conn.prepare("SELECT * FROM sections WHERE id = ?")?;
    stmt.query_row([section_id], |row| Section::try_from(row))
}

pub fn update_section(section: &Section) -> Result<()> {
    let conn = get_connection();
    let old_section = read_section(section.id())?;
    let index_stmt = &mut String::new();

    if section.index() != old_section.index() {
        index_stmt.push_str(&format!(", i = {}", section.index()));
        match section.index().cmp(&old_section.index()) {
            Ordering::Greater => {
                conn.execute(
                    "UPDATE sections SET i = i - 1
                    WHERE i > ?1 AND i <= ?2",
                    (old_section.index(), section.index()),
                )?;
            }
            Ordering::Less => {
                conn.execute(
                    "UPDATE sections SET i = i + 1
                    WHERE i < ?1 AND i >= ?2",
                    (old_section.index(), section.index()),
                )?;
            }
            Ordering::Equal => {}
        }
    }

    conn.execute(
        &format!(
            "UPDATE sections SET
            name = ?2, project = ?3, i = ?4 {index_stmt} WHERE id = ?1"
        ),
        (
            section.id(),
            section.name(),
            section.project(),
            section.index(),
        ),
    )?;
    Ok(())
}

pub fn delete_section(section_id: i64) -> Result<()> {
    let conn = get_connection();
    // Notify: Not return error when id not exists
    conn.execute("DELETE FROM sections WHERE id = ?", (section_id,))?;
    conn.execute("DELETE FROM tasks WHERE section = ?", (section_id,))?;
    Ok(())
}

fn new_index(project_id: i64) -> i32 {
    let conn = get_connection();
    let mut stmt = conn
        .prepare("SELECT i FROM sections WHERE project = ? ORDER BY i DESC")
        .expect("Failed to find new index");
    let first_row = stmt.query_row([project_id], |row| row.get::<_, i32>(0));
    match first_row {
        Ok(first_row) => first_row + 1,
        Err(_) => 0,
    }
}
