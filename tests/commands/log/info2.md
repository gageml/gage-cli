# Log info (working tests for review-2)

Generate a log with a test task.

    >>> cd(make_temp_dir())

    >>> write_file("test.py", """
    ... from inspect_ai import task, Task
    ...
    ... @task
    ... def test():
    ...     return Task(model="mockllm/model")
    ... """)

    >>> run("gage run test -i '' -y", quiet=True)

    >>> ls("logs")  # +parse
    {timestamp}_{task}_{log_id}.eval

Show log info.

    >>> run(f"gage log info2 {log_id}")  # +parse +table
    ╭─────────┬────────────────────────╮
    │ Log     │ {log_id_}              │
    ├─────────┼────────────────────────┤
    │ Task    │ test                   │
    │ Created │ now                    │
    ╰─────────┴────────────────────────╯

    >>> assert log_id_ == log_id
