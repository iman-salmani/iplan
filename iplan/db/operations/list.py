from typing import Mapping

from iplan.db.manager import connect_database
from iplan.db.models.list import List

def create_list(name: str, project_id: int) -> List:
    position = find_new_list_position(project_id)
    connection, cursor = connect_database()
    cursor.execute(
        f"""INSERT INTO lists(name, project, position)
        VALUES ('{name}', {project_id}, {position})"""
    )
    connection.commit()
    return read_list(cursor.lastrowid)

def read_lists(project_id: int) -> Mapping[List, list]:
    query = f"""SELECT * FROM lists WHERE
    project = {project_id} ORDER BY position ASC"""
    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(List.new_from_record, records)

def read_list(list_id: int) -> List:
    connection, cursor = connect_database()
    return List.new_from_record(
        cursor.execute(
            f"SELECT * FROM lists WHERE id = {list_id}"
        ).fetchone()
    )

def update_list(_list: List) -> None:
    connection, cursor = connect_database()
    cursor.execute(
        f"""UPDATE lists SET
        name = '{_list.name}',
        project = {_list.project},
        position = {_list.position}
        WHERE id = {_list._id}"""
    )
    connection.commit()

def delete_list(list_id: int) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM lists WHERE id = {list_id}")
    cursor.execute(f"DELETE FROM tasks WHERE list = {list_id}")
    connection.commit()

def find_new_list_position(project_id) -> int:
    connection, cursor = connect_database()
    first_record = cursor.execute(
        f"""SELECT * FROM lists WHERE
        project = {project_id} ORDER BY position DESC"""
    ).fetchone()
    if not first_record:
        return 0
    return first_record[2] + 1

