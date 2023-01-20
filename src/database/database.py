import sqlite3
import os
from dataclasses import dataclass
from typing import Optional
from datetime import datetime, date
from gi.repository import GLib

class Database:
    path = os.path.join(GLib.get_user_data_dir(), "database.db")
    # path = "/home/iman/.cache/ir.imansalmani.iplan/database.db"

    def __init__(self) -> None:
        if os.path.isfile(self.path):
            self.connect()
        else:
            self.connect()

            self.cursor.execute(
                """
                CREATE TABLE "tasks" (
                "id"	  INTEGER NOT NULL,
                "name"	  TEXT NOT NULL,
                "done"	  INTEGER NOT NULL,
                "project" INTEGER NOT NULL,
                "times"   TEXT NOT NULL,
                "position"   INTEGER NOT NULL DEFAULT 0,
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
        self.connection = sqlite3.connect(self.path)
        self.cursor = self.connection.cursor()


@dataclass
class Project:
    id: int
    name: str
    archive: bool

    def get_duration(self):
        tasks = TasksData().all(show_completed_tasks=True, project=self)
        duration = 0
        for task in tasks:
            duration += task.get_duration()

        return duration

    def get_duration_table(self) -> dict[date, int]:
        table = {}

        tasks = TasksData().all(show_completed_tasks=True, project=self)
        for task in tasks:
            for time in task.times.split(";")[0:-1]:
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
        if archive:
            _filter = "ORDER BY archive ASC"
        else:
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
        self.cursor.execute(f"DELETE FROM tasks WHERE project = {_id}")
        self.connection.commit()

    def first(self) -> Project:
        all = self.all()
        if not all:
            return self.all(archive=True)[0]
        return all[0]

    def search(self, text, archive=False) -> list[Project]:
        all = self.all(archive)

        result = []
        for project in all:
            if project.name.lower().find(text) != -1:
                result.append(project)

        return result


@dataclass
class Task:
    id: int
    name: str
    done: bool
    project: Optional[int]
    times: str
    position: int

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


class TasksData(Database):
    def __init__(self) -> None:
        super().__init__()

    def record_to_task(self, record) -> Project:
        """convert database record to Task dataclass"""
        return Task(
            id=record[0],
            name=record[1],
            done=record[2],
            project=record[3],
            times=record[4],
            position=record[5]
        )

    def all(self, show_completed_tasks=False, project=None) -> list[Task]:
        if project == None:
            project_filter = "project is NULL"
        else:
            project_filter = f"project = {project.id}"

        completed_tasks_filter = ""
        if show_completed_tasks == False:
            completed_tasks_filter = f"AND done = {False} "

        query = f"SELECT * FROM tasks WHERE {project_filter} {completed_tasks_filter} ORDER BY position ASC"
        tasks = []
        for record in self.cursor.execute(query).fetchall():
            tasks.append(self.record_to_task(record))
        return tasks

    def delete(self, _id) -> None:
        self.cursor.execute(f"DELETE FROM tasks WHERE id = {_id}")
        self.connection.commit()

    def update(self, task: Task) -> None:
        self.cursor.execute(
            f"UPDATE tasks SET name = '{task.name}', done = {task.done}, project = {task.project}, times = '{task.times}', position = {task.position} WHERE id = {task.id}"
        )
        self.connection.commit()

    def add(self, name, project_id: int) -> Task:
        position = self.cursor.execute(f"SELECT * FROM tasks WHERE project = {project_id} ORDER BY position DESC").fetchone()[5] + 1
        # 👆️ explain:
        # - get all tasks in project order by position big to low
        # - fetch first one
        # - add one to position
        self.cursor.execute(
            f"INSERT INTO tasks(name, done, project, times, position) VALUES ('{name}', 0, {project_id}, '', {position})"
        )
        self.connection.commit()

        return Task(self.cursor.lastrowid, name, False, project_id, "", position)

    def search(self, text) -> list[Task]:
        query = f"SELECT * FROM tasks WHERE name LIKE '%{text}%'"
        result = self.cursor.execute(query).fetchall()
        tasks = []
        for row in result:
            tasks.append(self.record_to_task(row))
        return tasks

    def get(self, task_id: int) -> Task:
        return self.record_to_task(
            self.cursor.execute(
                f"SELECT * FROM tasks WHERE id = {task_id}"
            ).fetchone()
        )

