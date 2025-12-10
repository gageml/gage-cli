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

Create a "stop file" to prevent Gage from scanning for files above the
temp dir.

    >>> touch(".venv")  # Gage treats this as a "stop file"

    >>> ls()
    .venv

Log dir and Config show default values that are used.

    >>> run("gage status")  # +wildcard -space
    ╭──────────────────────┬───────────╮
    │ gage version         │ 0.2...    │
    │ gage_inspect version │ 0.2...    │
    │ inspect_ai version   │ 0.3...    │
    │ Python version       │ 3...      │
    │ .env                 │           │
    │ Log dir              │ logs      │
    │ Config               │ gage.toml │
    │ Active profile       │           │
    ╰──────────────────────┴───────────╯

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

    >>> run("gage status")  # +wildcard -space
    ╭──────────────────────┬───────────╮
    │ gage version         │ 0.2...    │
    │ gage_inspect version │ 0.2...    │
    │ inspect_ai version   │ 0.3...    │
    │ Python version       │ 3...      │
    │ .env                 │ .env      │
    │ Log dir              │ logs2     │
    │ Config               │ gage.toml │
    │ Active profile       │ dev       │
    ╰──────────────────────┴───────────╯

Use `--verboase`.

    >>> run("gage status --verbose")  # +wildcard -space
    ╭──────────────────────┬───────────────────────────────────╮
    │ gage version         │ 0.2...                            │
    │ gage_inspect version │ 0.2...                            │
    │ gage_inspect path    │ ...                               │
    │ inspect_ai version   │ 0.3...                            │
    │ inspect_ai path      │ ...                               │
    │ Python version       │ 3...                              │
    │ Python executable    │ ...                               │
    │ Python sys path      │ ...                               │
    │ .env                 │ .env                              │
    │ Log dir              │ logs2                             │
    │ Config               │ gage.toml                         │
    │ Active profile       │ dev                               │
    ╰──────────────────────┴───────────────────────────────────╯
