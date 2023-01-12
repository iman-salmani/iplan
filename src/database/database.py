import sqlite3
import os
from dataclasses import dataclass
from typing import Optional
from datetime import datetime, date


class Database:
    def __init__(self) -> None:
        if os.path.isfile(".local/database.db"):
            self.connect()
        else:
            self.connect()

            self.cursor.execute(
                """
                CREATE TABLE "todos" (
                "id"	  INTEGER NOT NULL,
                "name"	  TEXT NOT NULL,
                "done"	  INTEGER NOT NULL,
                "project" INTEGER,
                "times"   TEXT NOT NULL,
                PRIMARY KEY("id" AUTOINCREMENT)
            );"""
            )

            self.cursor.execute(
                """
                CREATE TABLE "projects" (
                "id"	  INTEGER NOT NULL,
                "name"	  TEXT NOT NULL,
                "archive" INTEGER NOT NULL,
                PRIMARY KEY("id" AUTOINCREMENT)
            );"""
            )

            self.cursor.execute(
                f"INSERT INTO projects(name, archive) VALUES ('Personal', 0)"
            )

            self.connection.commit()

    def connect(self) -> None:
        self.connection = sqlite3.connect(".local/database.db")
        self.cursor = self.connection.cursor()


@dataclass
class Project:
    id: int
    name: str
    archive: bool

    def get_duration(self):
        todos = TodosData().all(show_completed_tasks=True, project=self)
        duration = 0
        for todo in todos:
            duration += todo.get_duration()

        return duration

    def get_duration_table(self) -> dict[date, int]:
        table = {}

        todos = TodosData().all(show_completed_tasks=True, project=self)
        for todo in todos:
            for time in todo.times.split(";")[0:-1]:
                _datetime = float(time.split(",")[0])
                _date = datetime.fromtimestamp(_datetime).date()
                duration = int(time.split(",")[1])

                if _date in table.keys():
                    table[_date] += duration
                else:
                    table[_date] = duration

        return table

    def duration_to_text(self, duration):
        duration_minute, duration_second = divmod(duration, 60)
        duration_hour, duration_minute = divmod(duration_minute, 60)

        text = ""
        if duration_hour != 0:
            text = "{:d}:{:02d}:{:02d}".format(
                duration_hour, duration_minute, duration_second
            )
        else:
            text = "{:d}:{:02d}".format(duration_minute, duration_second)

        return text


class ProjectsData(Database):
    def __init__(self) -> None:
        super().__init__()

    def record_to_project(self, record) -> Project:
        """convert database record to Project dataclass"""
        return Project(id=record[0], name=record[1], archive=record[2])

    def get(self, project_id: int) -> Project:
        return self.record_to_project(
            self.cursor.execute(
                f"SELECT * FROM projects WHERE id = {project_id}"
            ).fetchone()
        )

    def all(self, archive=False) -> list[Project]:
        _filter = ""
        if not archive:
            _filter = "WHERE archive = False"

        result = self.cursor.execute(f"SELECT * FROM projects {_filter}").fetchall()

        projects = []
        for record in result:
            projects.append(self.record_to_project(record))

        return projects

    def add(self, name) -> int:
        self.cursor.execute(f"INSERT INTO projects(name, archive) VALUES ('{name}', 0)")
        self.connection.commit()
        return self.cursor.lastrowid

    def update(self, project: Project) -> None:
        self.cursor.execute(
            f"UPDATE projects SET name = '{project.name}', archive = {project.archive} WHERE id = {project.id}"
        )
        self.connection.commit()

    def archive(self, _id: int, archive: bool) -> None:
        self.cursor.execute(f"UPDATE projects SET archive = {archive} WHERE id = {_id}")
        self.connection.commit()

    def delete(self, _id) -> None:
        self.cursor.execute(f"DELETE FROM projects WHERE id = {_id}")
        self.cursor.execute(f"DELETE FROM todos WHERE project = {_id}")
        self.connection.commit()

    def first(self) -> Project:
        all = self.all()
        if not all:
            return self.all(archive=True)[0]
        return all[0]

    def search(self, text, archive) -> list[Project]:
        all = self.all(archive)

        result = []
        for project in all:
            if project.name.lower().find(text) != -1:
                result.append(project)

        return result


@dataclass
class Todo:
    id: int
    name: str
    done: bool
    project: Optional[int]
    times: str

    def get_last_time(self) -> Optional[list[float, int]]:
        if self.times:
            time = self.times.split(";")[0:-1][-1].split(",")
            return [float(time[0]), int(time[1])]

    def get_duration(self):
        duration = 0
        for time in self.times.split(";")[0:-1]:
            duration += int(time.split(",")[1])
        return duration

    def get_duration_text(self):
        duration = self.get_duration()
        text = self.duration_to_text(duration)

        return text

    def duration_to_text(self, duration):
        duration_minute, duration_second = divmod(duration, 60)
        duration_hour, duration_minute = divmod(duration_minute, 60)

        text = ""
        if duration_hour != 0:
            text = "{:d}:{:02d}:{:02d}".format(
                duration_hour, duration_minute, duration_second
            )
        else:
            text = "{:d}:{:02d}".format(duration_minute, duration_second)

        return text


class TodosData(Database):
    def __init__(self) -> None:
        super().__init__()

    def record_to_todo(self, record) -> Project:
        """convert database record to Todo dataclass"""
        return Todo(
            id=record[0],
            name=record[1],
            done=record[2],
            project=record[3],
            times=record[4],
        )

    def all(self, show_completed_tasks=False, project=None) -> list[Todo]:
        if project == None:
            project_filter = "project is NULL"
        else:
            project_filter = f"project = {project.id}"

        completed_tasks_filter = ""
        completed_tasks_order = ""
        if show_completed_tasks == True:
            completed_tasks_order = "ORDER BY done DESC"
        else:
            completed_tasks_filter = f"AND done = {show_completed_tasks} "

        query = f"SELECT * FROM todos WHERE {project_filter} {completed_tasks_filter} {completed_tasks_order}"
        todos = []
        for record in self.cursor.execute(query).fetchall():
            todos.append(self.record_to_todo(record))
        todos.reverse()
        return todos

    def delete(self, _id) -> None:
        self.cursor.execute(f"DELETE FROM todos WHERE id = {_id}")
        self.connection.commit()

    def update(self, todo: Todo) -> None:
        self.cursor.execute(
            f"UPDATE todos SET name = '{todo.name}', done = {todo.done}, project = {todo.project}, times = '{todo.times}' WHERE id = {todo.id}"
        )
        self.connection.commit()

    def add(self, name, project_id="NULL") -> Todo:
        self.cursor.execute(
            f"INSERT INTO todos(name, done, project, times) VALUES ('{name}', 0, {project_id}, '')"
        )
        self.connection.commit()

        return Todo(self.cursor.lastrowid, name, False, project_id, "")

