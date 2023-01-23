from typing import Mapping

from iplan.db.manager import connect_database
from iplan.db.models.project import Project

def create_project(name: str) -> Project:
    connection, cursor = connect_database()
    cursor.execute(f"INSERT INTO projects(name) VALUES ('{name}')")
    connection.commit()
    return read_project(cursor.lastrowid)

def read_projects(archive: bool=False) -> Mapping[Project, list]:
    _filter = "WHERE archive = False"
    if archive:
        _filter = "ORDER BY archive ASC"
    query = f"SELECT * FROM projects {_filter}"

    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(Project.new_from_record, records)

def read_project(project_id: int) -> Project:
    connection, cursor = connect_database()
    return Project.new_from_record(
        cursor.execute(
            f"SELECT * FROM projects WHERE id = {project_id}"
        ).fetchone()
    )

def update_project(project: Project) -> None:
    connection, cursor = connect_database()
    cursor.execute(
        f"""UPDATE projects SET
        name = '{project.name}',
        archive = {project.archive}
        WHERE id = {project._id}"""
    )
    connection.commit()

def delete_project(project_id: int) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM projects WHERE id = {project_id}")
    cursor.execute(f"DELETE FROM tasks WHERE project = {project_id}")
    connection.commit()

def search_projects(text: str, archive=False) -> Mapping[Project, list]:
    _filter = "AND archive = False"
    if archive:
        _filter = "AND ORDER BY archive ASC"
    query = f"SELECT * FROM projects WHERE name LIKE '%{text}%' {_filter}"

    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(Project.new_from_record, records)

