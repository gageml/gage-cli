# `gage status` command

    >>> run("gage status --help")  # +diff
    Show project status
    ⤶
    Usage: gage status [OPTIONS]
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -v, --verbose            Show more detail
      -h, --help               Print help

## Empty directory

Create an empty directory.

    >>> cd(make_temp_dir())

    >>> ls()
    <empty>

Log dir and Config show default values that are used.

    >>> run("gage status")
    ╭──────────────────────┬────────────╮
    │ gage version         │ 0.2.0-dev1 │
    │ gage_inspect version │ 0.2.0.dev2 │
    │ inspect_ai version   │ 0.3.150    │
    │ .env                 │            │
    │ Log dir              │ logs       │
    │ Config               │ gage.toml  │
    │ Active profile       │            │
    ╰──────────────────────┴────────────╯

## Profile

Create a directory containing `gage.toml` with a `dev` profile. The
profile defines a non-default log dir.

    >>> cd(make_temp_dir())

    >>> write_file("gage.toml", """
    ... [profiles.dev]
    ... log_dir = "logs2"
    ... """)

    >>> ls()
    gage.toml

Activate the environment.

    >>> run("gage profile use dev")
    ╭────────────────┬───────╮
    │ Active profile │ dev   │
    ├────────────────┼───────┤
    │ Log dir        │ logs2 │
    ╰────────────────┴───────╯

Gage creates `.env`.

    >>> ls()
    .env
    gage.toml

Status shows configured log dir.

    >>> run("gage status")
    ╭──────────────────────┬────────────╮
    │ gage version         │ 0.2.0-dev1 │
    │ gage_inspect version │ 0.2.0.dev2 │
    │ inspect_ai version   │ 0.3.150    │
    │ .env                 │ .env       │
    │ Log dir              │ logs2      │
    │ Config               │ gage.toml  │
    │ Active profile       │ dev        │
    ╰──────────────────────┴────────────╯
