from typing import Mapping

from iplan.db.manager import connect_database
from iplan.db.models.project import Project

def create_project(name: str) -> Project:
    index = find_new_project_index()
    connection, cursor = connect_database()
    cursor.execute(f"INSERT INTO projects(name, i) VALUES ('{name}', {index})")
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

def update_project(project: Project, move_index=False) -> None:
    connection, cursor = connect_database()
    index_statement = ''

    if move_index:
        index_statement = f", i = {project.index}"
        old_project = read_project(project._id)

        projects_between = []
        step = 0
        range_condition = []    # start after or before old index to new index

        if old_project.index < project.index:
            range_condition = [">", "<="]
            step = -1
        elif old_project.index > project.index:
            range_condition = ["<", ">="]
            step = +1

        projects_between = cursor.execute(f"""SELECT * FROM projects WHERE
            i {range_condition[0]} {old_project.index} AND
            i {range_condition[1]} {project.index}
            """).fetchall()

        for record in projects_between:
            cursor.execute(
                f"""UPDATE projects SET
                i = {record[3]+step}
                WHERE id = {record[0]}"""
            )

    cursor.execute(
        f"""UPDATE projects SET
        name = '{project.name}',
        archive = {project.archive}
        {index_statement}
        WHERE id = {project._id}"""
    )
    connection.commit()

def delete_project(project: Project) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM projects WHERE id = {project._id}")
    cursor.execute(f"DELETE FROM lists WHERE project = {project._id}")
    cursor.execute(f"DELETE FROM tasks WHERE project = {project._id}")

    # decrease lower projects
    upper_projects = cursor.execute(
        f"SELECT * FROM projects WHERE i > {project.index}"
        ).fetchall()
    for record in upper_projects:
            cursor.execute(
                f"""UPDATE projects SET
                i = {record[3]-1}
                WHERE id = {record[0]}"""
            )

    connection.commit()

def search_projects(text: str, archive=False) -> Mapping[Project, list]:
    _filter = "AND archive = False"
    if archive:
        _filter = "AND ORDER BY archive ASC"
    query = f"SELECT * FROM projects WHERE name LIKE '%{text}%' {_filter}"

    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(Project.new_from_record, records)

def find_new_project_index() -> int:
    connection, cursor = connect_database()
    last_index = cursor.execute("SELECT i FROM projects ORDER BY i DESC").fetchone()
    if not last_index:
        return 0
    return last_index[0] + 1

