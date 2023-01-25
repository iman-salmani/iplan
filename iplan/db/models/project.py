from dataclasses import dataclass
from datetime import datetime, date

from iplan.db.operations.task import read_tasks


@dataclass
class Project:
    _id: int
    name: str
    archive: bool

    def new_from_record(record: list):
        return Project(
            _id=record[0],
            name=record[1],
            archive=record[2]
        )

    def get_duration(self):
        tasks = read_tasks(project_id=self._id)
        duration = 0
        for task in tasks:
            duration += task.get_duration()
        return duration

    def get_duration_table(self) -> dict[date, int]:
        table = {}

        tasks = read_tasks(project_id=self._id)
        for task in tasks:
            for time in task.duration.split(";")[0:-1]:
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

