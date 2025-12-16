# `gage log restore` command

    >>> run("gage log restore2 --help")
    Restore deleted logs
    ⤶
    Usage: gage log restore2 [OPTIONS] [SPEC]...
    ⤶
    Arguments:
      [SPEC]...
              Log(s) to restsore.
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

Restore the run using the table index.

    >>> run("gage log restore2 1 -y", cols=72)  # +parse +table
    ┌  Restore logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  1 log restored

    >>> assert id_ == id

    >>> run("gage list --deleted", cols=72)  # +parse +table
    No logs found

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id

## Restoring to an existing file

In the case where a file would be overwritten, Gage shows an error
messages.

Copy the log file to create a deleted version.

    >>> log_filename = ls_list("logs")[0]
    >>> copy(f"logs/{log_filename}", f"logs/{log_filename}.deleted")

The logs exists both as deleted and non-deleted.

    >>> run("gage list --deleted", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

    >>> assert id_ == id

Try to restore the log.

    >>> run(f"gage log restore2 {id} -y", cols=72)  # +parse +table
    ┌  Restore logs
    │
    ├  ╭────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │  │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    │  ├────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │  │ {id_}  │ test │ run  │ success │ echo/echo │         │ now     │
    │  ╰────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯
    │
    └  Error
    ⤶
    CONFLICT: log {full_id_} exists and would be overwritten (see logs/{path_})
    <1>

    >>> assert id_ == id
    >>> assert full_id_.startswith(id)
    >>> assert path_ == log_filename
