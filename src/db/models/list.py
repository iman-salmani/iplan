from dataclasses import dataclass


@dataclass
class List:
    _id: int
    name: str
    project: int
    index: int

    def new_from_record(record: list):
        return List(_id=record[0], name=record[1], project=record[2], index=record[3])
