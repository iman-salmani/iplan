import sqlite3
import os
from gi.repository import GLib


PATH = os.path.join(GLib.get_user_data_dir(), "data.db")
if "INSIDE_GNOME_BUILDER" in os.environ:
    PATH = os.path.join(GLib.get_user_cache_dir(), "data.db")

def check_database() -> None:
    if not os.path.isfile(PATH):
        create_tables()

def connect_database() -> (sqlite3.Connection, sqlite3.Cursor):
    # TODO: test yield with try and except and finally
    connection = sqlite3.connect(PATH)
    cursor = connection.cursor()
    return connection, cursor

def create_tables() -> None:
    connection, cursor = connect_database()
    cursor.execute(
        """
        CREATE TABLE "projects" (
        "id"	    INTEGER NOT NULL,
        "name"	    TEXT NOT NULL,
        "archive"   INTEGER NOT NULL DEFAULT 0,
        "i"  INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY("id" AUTOINCREMENT)
        );
        """
    )

    cursor.execute(
        """
        CREATE TABLE "lists" (
        "id"        INTEGER NOT NULL,
        "name"      TEXT NOT NULL,
        "project"   INTEGER NOT NULL,
        "position"  INTEGER NOT NULL,
        PRIMARY KEY("id" AUTOINCREMENT)
        );
        """
    )

    cursor.execute(
        """
        CREATE TABLE "tasks" (
        "id"	  INTEGER NOT NULL,
        "name"	  TEXT NOT NULL,
        "done"	  INTEGER NOT NULL DEFAULT 0,
        "project" INTEGER NOT NULL,
        "list" INTEGER NOT NULL,
        "duration"   TEXT NOT NULL DEFAULT '',
        "position"   INTEGER NOT NULL,
        PRIMARY KEY("id" AUTOINCREMENT)
        );
        """
    )

    cursor.execute(
        f"INSERT INTO projects(name) VALUES ('Personal')"
    )

    cursor.execute(
        f"INSERT INTO lists(name, project, position) VALUES ('Tasks', 1, 0)"
    )

    connection.commit()

