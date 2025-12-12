# Profile env

Create a config file in a new directory.

    >>> cd(make_temp_dir())

    >>> write_file("gage.toml", """
    ... [profiles.foo]
    ... env.FOO = "123"
    ... env.BAR = "abc"
    ... """)

Create a task that writes env to a file.

    >>> write_file("test.py", """
    ... import os
    ... from inspect_ai import task, Task
    ...
    ... @task
    ... def test():
    ...     with open("env", "w") as f:
    ...         f.write("FOO={}\\n".format(os.getenv("FOO")))
    ...         f.write("BAR={}\\n".format(os.getenv("BAR")))
    ...     return Task(model="mockllm/model")
    ... """)

Run the task without an active profile.

    >>> run("gage profile status")
    GAGE_PROFILE not defined - no profile in use
    <1>

    >>> run("gage run test -i '' -y", quiet=True)

Env is not set.

    >>> cat("env")
    FOO=None
    BAR=None

Run the task with an activated profile.

    >>> run("gage profile use foo")
    ╭────────────────┬─────╮
    │ Active profile │ foo │
    ├────────────────┼─────┤
    │ BAR            │ abc │
    │ FOO            │ 123 │
    ╰────────────────┴─────╯

    >>> run("gage run test -i '' -y", quiet=True)

Env is set.

    >>> cat("env")
    FOO=123
    BAR=abc
