from dataclasses import dataclass


@dataclass
class Task:
    _id: int
    name: str
    done: bool
    project: int
    _list: int
    duration: str
    position: int
    suspended: bool

    def new_from_record(record: list):
        return Task(
            _id=record[0],
            name=record[1],
            done=record[2],
            project=record[3],
            _list=record[4],
            duration=record[5],
            position=record[6],
            suspended=record[7],
        )

    def get_last_time(self) -> list[float, int] | None:
        if self.duration:
            time = self.duration.split(";")[0:-1][-1].split(",")
            return [float(time[0]), int(time[1])]

    def get_duration_text(self) -> str:
        duration = self.get_duration()
        return self.duration_to_text(duration)

    def get_duration(self) -> int:
        duration = 0
        for time in self.duration.split(";")[0:-1]:
            duration += int(time.split(",")[1])
        return duration

    def duration_to_text(self, duration) -> int:
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
