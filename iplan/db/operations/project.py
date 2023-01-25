from typing import Mapping

from iplan.db.manager import connect_database
from iplan.db.models.project import Project

def create_project(name: str) -> Project:
    position = find_new_project_position()
    connection, cursor = connect_database()
    cursor.execute(f"INSERT INTO projects(name, position) VALUES ('{name}', {position})")
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

def update_project(project: Project, move_position=False) -> None:
    connection, cursor = connect_database()
    position_statement = ''

    if move_position:
        position_statement = f", position = {project.position}"
        old_project = read_project(project._id)

        increase_position_projects = []
        decrease_position_projects = []

        if old_project.position < project.position:
            decrease_position_projects = cursor.execute(f"""SELECT * FROM projects WHERE
                position > {old_project.position} AND
                position <= {project.position}
                """).fetchall()
        elif old_project.position > project.position:
            increase_position_projects = cursor.execute(f"""SELECT * FROM projects WHERE
                position >= {project.position} AND
                position < {old_project.position}
                """).fetchall()

        for record in increase_position_projects:
            cursor.execute(
                f"""UPDATE projects SET
                position = {record[3]+1}
                WHERE id = {record[0]}"""
            )

        for record in decrease_position_projects:
            cursor.execute(
                f"""UPDATE projects SET
                position = {record[3]-1}
                WHERE id = {record[0]}"""
            )

    cursor.execute(
        f"""UPDATE projects SET
        name = '{project.name}',
        archive = {project.archive}
        {position_statement}
        WHERE id = {project._id}"""
    )
    connection.commit()

def delete_project(project_id: int) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM projects WHERE id = {project_id}")
    cursor.execute(f"DELETE FROM lists WHERE project = {project_id}")
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

def find_new_project_position() -> int:
    connection, cursor = connect_database()
    first_record = cursor.execute(
        f"""SELECT * FROM projects ORDER BY position DESC"""
    ).fetchone()
    if not first_record:
        return 0
    return first_record[3] + 1

