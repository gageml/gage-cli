# Delete logs

    >>> run("gage log delete2 --help")  # +diff
    Delete logs
    ⤶
    Usage: gage log delete2 [OPTIONS] [SPEC]...
    ⤶
    Arguments:
      [SPEC]...
              Log(s) to delete.
    ⤶
              SPEC may be a log ID, index position, or index position
              range in the form START:END.
    ⤶
    Options:
          --log-dir <LOG_DIR>
              Log directory
    ⤶
      -a, --all
              Delete all logs
    ⤶
      -y, --yes
              Don't prompt for dialog
    ⤶
      -p, --permanent
              Permanently delete logs
    ⤶
      -h, --help
              Print help (see a summary with '-h')

## Example

Generate three runs.

    >>> cd(make_temp_dir())

    >>> write_file("task.py", """
    ... from inspect_ai import task
    ... from gage_inspect.task import Echo
    ...
    ... @task
    ... def test():
    ...     return Echo()
    ... """)

    >>> run("gage run test -i foo -y", quiet=True)
    >>> run("gage run test -i bar -y", quiet=True)
    >>> run("gage run test -i baz -y", quiet=True)

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id1}  │ test │ run  │ success │ echo/echo │         │ now     │
    │ 2 │ {id2}  │ test │ run  │ success │ echo/echo │         │ now     │
    │ 3 │ {id3}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

Delete log 1.

    >>> run("gage log delete2 1 -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log deleted

    >>> assert id_ == id1

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id1_} │ test │ run  │ success │ echo/echo │         │ now     │
    │ 2 │ {id2_} │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id1_ == id2
    >>> assert id2_ == id3

Delete the second log by ID.

    >>> run(f"gage log delete2 {id2} -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log deleted

    >>> assert id_ == id2

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id3

Deleted runs:

    >>> run("gage list -d", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id1_} │ test │ run  │ success │ echo/echo │         │ now     │
    │ 2 │ {id2_} │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id1_ == id1
    >>> assert id2_ == id2

## Delete by range

Delete `2:3`

    >>> run("gage log restore --all -y", quiet=True)
    >>> run("gage log delete2 2:3 -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id1_} │ test │ run  │ success │ echo/echo │         │ now     │
    │  │ {id2_} │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  2 logs deleted

    >>> assert id1_ == id2
    >>> assert id2_ == id3

Delete `:2`

    >>> run("gage log restore --all -y", quiet=True)
    >>> run("gage log delete2 1:2 -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id1_} │ test │ run  │ success │ echo/echo │         │ now     │
    │  │ {id2_} │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  2 logs deleted

    >>> assert id1_ == id1
    >>> assert id2_ == id2

Delete `3:`.

    >>> run("gage log restore --all -y", quiet=True)
    >>> run("gage log delete2 3: -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id1_} │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log deleted

    >>> assert id1_ == id3

## Delete already deleted file

This tests an edge case where the same log exists as deleted and
non-deleted.

Restore everything.

    >>> run("gage log restore --all -y", quiet=True)

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │ 2 │ {}     │ test │ run  │ success │ echo/echo │         │ now     │
    │ 3 │ {}     │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id1

    >>> run("gage list --deleted")
    No logs found

Copy first log as `.deleted`.

    >>> id1_filename = [path for path in ls_list("logs") if id1 in path][0]

    >>> copy(f"logs/{id1_filename}", f"logs/{id1_filename}.deleted")

List deleted logs.

    >>> run("gage list --deleted", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id1

Try to delete the log.

    >>> run(f"gage log delete2 {id1} -y", cols=72)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  Error
    ⤶
    CONFLICT: log {full_id_} already deleted (see logs/{path_}.deleted)
    <1>

    >>> assert id_ == id1
    >>> assert full_id_.startswith(id1)
    >>> assert path_ == id1_filename

The log is not deleted.

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │ 2 │ {}     │ test │ run  │ success │ echo/echo │         │ now     │
    │ 3 │ {}     │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id1
