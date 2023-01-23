from typing import Mapping

from iplan.db.manager import connect_database
from iplan.db.models.task import Task

def create_task(name: str, project_id: int, list_id: int) -> Task:
    position = find_new_task_position(list_id)
    connection, cursor = connect_database()
    cursor.execute(
        f"""INSERT INTO tasks(name, project, list, position)
        VALUES ('{name}', {project_id}, {list_id}, {position})"""
    )
    connection.commit()
    return read_task(cursor.lastrowid)

def read_tasks(
        project_id: int,
        list_id: int= None,
        completed_tasks: bool=False
        ) -> Mapping[Task, list]:

    _filter = f"project = {project_id}"
    if list_id != None:
        _filter += f" AND list = {list_id}"
    if completed_tasks == False:
        _filter += f" AND done = {False}"
    query = f"SELECT * FROM tasks WHERE {_filter} ORDER BY position ASC"

    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(Task.new_from_record, records)

def read_task(task_id: int) -> Task:
    connection, cursor = connect_database()
    return Task.new_from_record(
        cursor.execute(
            f"SELECT * FROM tasks WHERE id = {task_id}"
        ).fetchone()
    )

def update_task(task: Task) -> None:
    connection, cursor = connect_database()
    cursor.execute(
        f"""UPDATE tasks SET
        name = '{task.name}',
        done = {task.done},
        project = {task.project},
        list = {task._list},
        duration = '{task.duration}',
        position = {task.position}
        WHERE id = {task._id}"""
    )
    connection.commit()

def delete_task(task_id: int) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM tasks WHERE id = {task_id}")
    connection.commit()

def search_tasks(text: str) -> Mapping[Task, list]:
    query = f"SELECT * FROM tasks WHERE name LIKE '%{text}%'"
    connection, cursor = connect_database()
    records = cursor.execute(query).fetchall()
    return map(Task.new_from_record, records)

def find_new_task_position(list_id: int) -> int:
    connection, cursor = connect_database()
    first_record = cursor.execute(
        f"""SELECT * FROM tasks WHERE
        list = {list_id} ORDER BY position DESC"""
    ).fetchone()
    if not first_record:
        return 0
    return first_record[6] + 1

