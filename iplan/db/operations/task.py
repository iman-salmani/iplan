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
        list_id: int = None,
        done_tasks: bool = None
        ) -> Mapping[Task, list]:
    """if done_tasks be None it return both"""

    _filter = f"project = {project_id}"
    if list_id != None:
        _filter += f" AND list = {list_id}"
    if type(done_tasks) == bool:
        _filter += f" AND done = {done_tasks}"
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

def update_task(task: Task, move_position=False) -> None:
    connection, cursor = connect_database()
    position_statement = ''

    if move_position:
        position_statement = f", position = {task.position}"
        old_task = read_task(task._id)

        increase_position_tasks = []
        decrease_position_tasks = []

        if old_task._list != task._list:
            # Decrease tasks position in previous list
            decrease_position_tasks = cursor.execute(f"""SELECT * FROM tasks WHERE
                position > {old_task.position} AND
                list = {old_task._list}""").fetchall()

            # Increase tasks position in target list
            increase_position_tasks = cursor.execute(f"""SELECT * FROM tasks WHERE
                position >= {task.position} AND
                list = {task._list}""").fetchall()

        else:
            if old_task.position < task.position:
                decrease_position_tasks = cursor.execute(f"""SELECT * FROM tasks WHERE
                    position > {old_task.position} AND
                    position <= {task.position} AND
                    list = {task._list}
                    """).fetchall()
            elif old_task.position > task.position:
                increase_position_tasks = cursor.execute(f"""SELECT * FROM tasks WHERE
                    position >= {task.position} AND
                    position < {old_task.position} AND
                    list = {task._list}""").fetchall()

        for record in increase_position_tasks:
            cursor.execute(
                f"""UPDATE tasks SET
                position = {record[6]+1}
                WHERE id = {record[0]}"""
            )

        for record in decrease_position_tasks:
            cursor.execute(
                f"""UPDATE tasks SET
                position = {record[6]-1}
                WHERE id = {record[0]}"""
            )

    cursor.execute(
        f"""UPDATE tasks SET
        name = '{task.name}',
        done = {task.done},
        project = {task.project},
        list = {task._list},
        duration = '{task.duration}'
        {position_statement}
        WHERE id = {task._id}"""
    )
    connection.commit()

def delete_task(task_id: int) -> None:
    connection, cursor = connect_database()
    cursor.execute(f"DELETE FROM tasks WHERE id = {task_id}")
    connection.commit()

def search_tasks(text: str, done) -> Mapping[Task, list]:
    _filter = 'AND done = false'
    if done:
        _filter = ''
    query = f"SELECT * FROM tasks WHERE name LIKE '%{text}%' {_filter}"
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

