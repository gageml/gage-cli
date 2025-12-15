# Log dir

Run tests in an empty directory.

    >>> cd(make_temp_dir())
    >>> ls()
    <empty>

## Default

By default, the log directory is `logs`.

    >>> run("gage status --attr log_dir")  # +skip=CI - failing on macOS
    logs

TODO - macOS in CI showing /private/var/logs

## `INSPECT_LOG_DIR` env var

If `INSPECT_LOG_DIR` is specified, Gage uses it.

    >>> run("gage status --attr log_dir", {
    ...     "INSPECT_LOG_DIR": "from_env_var"
    ... })
    from_env_var

The var can be set in `.env`.

    >>> write_file(".env", "INSPECT_LOG_DIR=in_dot_env")

    >>> run("gage status --attr log_dir")
    in_dot_env

The process env overrides anything defined in `.env`.

    >>> run("gage status --attr log_dir", {
    ...     "INSPECT_LOG_DIR": "from_env_var_2"
    ... })
    from_env_var_2

## Gage profile

A log dir can be defined in a Gage profile.

    >>> write_file("gage.toml", """
    ... [profiles.default]
    ... log_dir = "in_profile"
    ... """)

    >>> run("gage profile list")
    ╭─────────┬─────────────╮
    │ Name    │ Description │
    ├─────────┼─────────────┤
    │ default │             │
    ╰─────────┴─────────────╯

A profile must be in use for its settings to take effect.

    >>> run("gage profile status")
    GAGE_PROFILE not defined - no profile in use
    <1>

    >>> run("gage status --attr log_dir")
    in_dot_env

    >>> run("gage profile use default")
    ╭────────────────┬────────────╮
    │ Active profile │ default    │
    ├────────────────┼────────────┤
    │ Log dir        │ in_profile │
    ╰────────────────┴────────────╯

    >>> cat(".env")
    INSPECT_LOG_DIR=in_dot_env
    GAGE_PROFILE=default

The value defined for `INSPECT_LOG_DIR` in `.env` takes precedence.

    >>> run("gage status --attr log_dir")
    in_dot_env

As does the process env.

    >>> run("gage status --attr log_dir", {
    ...     "INSPECT_LOG_DIR": "from_env_var_3"
    ... })
    from_env_var_3

Remove `INSPECT_LOG_DIR` from `.env`.

    >>> write_file(".env", "GAGE_PROFILE=default")

    >>> run("gage status --attr log_dir")
    in_profile

Again, process env takes prededence.

    >>> run("gage status --attr log_dir", {
    ...     "INSPECT_LOG_DIR": "from_env_var_4"
    ... })
    from_env_var_4

## Command option

The `--log-dir` option takes final precedence.

    >>> run("gage status --log-dir cmd_option --attr log_dir", {
    ...     "INSPECT_LOG_DIR": "from_env_var_5"
    ... })
    cmd_option

## Scenario

Write a config file that uses different log dirs.

    >>> write_file("gage.toml", """
    ... profiles.dev.log_dir = "logs-dev"
    ... profiles.test.log_dir = "logs-test"
    ... """)

Create a simple task.

    >>> write_file("task.py", """
    ... from inspect_ai import task, Task
    ...
    ... @task
    ... def test():
    ...     return Task(model="mockllm/model")
    ... """)

There are no logged runs at this point.

    >>> run("gage list")
    No logs found

    >>> ls(ignore="*.pyc")
    .env
    gage.toml
    task.py

Run task using the `dev` profile.

    >>> run("gage profile use dev")
    ╭────────────────┬──────────╮
    │ Active profile │ dev      │
    ├────────────────┼──────────┤
    │ Log dir        │ logs-dev │
    ╰────────────────┴──────────╯

    >>> run("gage run test --input 123 -y")
    ┌  Run task
    │
    ◇  Task:
    │  test
    │
    ◇  Input:
    │  123
    │
    ◇  Target:
    │  None
    │
    ●  Output:
    │
    │  Default output from mockllm/model
    │
    └  Done

The run is logged in `logs-dev`.

    >>> ls(ignore="*.pyc")  # +parse
    .env
    gage.toml
    logs-dev/{}_test_{}.eval
    task.py

Switch to `test` profile.

    >>> run("gage profile use test")
    ╭────────────────┬───────────╮
    │ Active profile │ test      │
    ├────────────────┼───────────┤
    │ Log dir        │ logs-test │
    ╰────────────────┴───────────╯

Run the test task.

    >>> run("gage run test --input 123 -y")
    ┌  Run task
    │
    ◇  Task:
    │  test
    │
    ◇  Input:
    │  123
    │
    ◇  Target:
    │  None
    │
    ●  Output:
    │
    │  Default output from mockllm/model
    │
    └  Done

The log is written to `logs-test`.

    >>> ls(ignore="*.pyc")  # +parse
    .env
    gage.toml
    logs-dev/{}_test_{}.eval
    logs-test/{}_test_{}.eval
    task.py
