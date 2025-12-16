# `gage log purge` command

    >>> run("gage log purge2 --help")
    Purge deleted logs
    ⤶
    Usage: gage log purge2 [OPTIONS] [SPEC]...
    ⤶
    Arguments:
      [SPEC]...
              Log(s) to permanently delete
    ⤶
              SPEC may be a log ID, index position, or index position
              range in the form START:END.
    ⤶
    Options:
          --log-dir <LOG_DIR>
              Log directory
    ⤶
      -a, --all
              Restore all deleted logs
    ⤶
      -y, --yes
              Don't prompt for dialog
    ⤶
      -h, --help
              Print help (see a summary with '-h')

## Example

Generate a run.

    >>> cd(make_temp_dir())

    >>> write_file("task.py", """
    ... from inspect_ai import task
    ... from gage_inspect.task import Echo
    ...
    ... @task
    ... def test():
    ...     return Echo()
    ... """)

    >>> run("gage run test -i '' -y", quiet=True)

    >>> run("gage list", cols=75)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id}   │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

Delete the run.

    >>> run("gage log delete2 --all -y", cols=75)  # +parse +table
    ┌  Delete logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log deleted

    >>> assert id_ == id

    >>> run("gage list --deleted", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id

Purge the run using the table index.

    >>> run("gage log purge2 1 -y", cols=72)  # +parse +table
    ┌  Restore logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log permanently deleted

    >>> assert id_ == id

    >>> run("gage list --deleted")
    No logs found

    >>> run("gage list")
    No logs found

    >>> ls("logs")
    <empty>
