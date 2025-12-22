# Review logs API

Create a logs.

    >>> cd(make_temp_dir())

    >>> write_file("task.py", """
    ... from inspect_ai import task, Task
    ... from gage_inspect.task import Echo
    ...
    ... @task
    ... def foo():
    ...     return Echo()
    ... """)

    >>> run("gage run foo -i 123 -y", quiet=True)

    >>> run("gage list", cols=72)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model     │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────┼─────────┼─────────┤
    │ 1 │ {id}   │ foo  │ run  │ success │ echo/echo │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────┴─────────┴─────────╯

Start `gage review`.

    >>> server = Server("gage log review2")

Wait a moment and get the port.

    >>> sleep(0.5)

    >>> server.print_output()  # +parse
    {}Listening on http://127.0.0.1:{port:d}

Gage uses ports from 3000 to 3010, binding to the first free port.

    >>> assert port >= 3000 and port <= 3010

Get the list of logs.

    >>> import requests

    >>> resp = requests.get(f"http://127.0.0.1:{port}/api/logs")
    >>> assert resp.ok

    >>> resp.json()  # +json +parse +diff
    [
      {
        "header": {
          "eval": {
            "config": {
              "continue_on_fail": false,
              "epochs": 1,
              "epochs_reducer": [
                "mean"
              ],
              "fail_on_error": true,
              "log_images": true,
              "log_realtime": false,
              "log_samples": true,
              "sandbox_cleanup": true,
              "score_display": true
            },
            "created": "{:isodate}",
            "dataset": {
              "sample_ids": [
                "run-input"
              ],
              "samples": 1,
              "shuffled": false
            },
            "eval_id": "{:id}",
            "model": "echo/echo",
            "model_args": {},
            "model_generate_config": {},
            "packages": {
              "inspect_ai": "0.3.{:d}"
            },
            "run_id": "{:id}",
            "tags": [
              "type:run"
            ],
            "task": "foo",
            "task_args": {},
            "task_args_passed": {},
            "task_attribs": {},
            "task_display_name": "foo",
            "task_file": "task.py",
            "task_id": "{:id}",
            "task_registry_name": "foo",
            "task_version": 0
          },
          "invalidated": false,
          "plan": {
            "config": {},
            "name": "plan",
            "steps": [
              {
                "params": {},
                "solver": "generate"
              }
            ]
          },
          "results": {
            "completed_samples": 1,
            "scores": [],
            "total_samples": 1
          },
          "stats": {
            "completed_at": "{:isodate}",
            "model_usage": {},
            "started_at": "{:isodate}"
          },
          "status": "success",
          "version": 2
        },
        "log_id": "{id_}"
      }
    ]

    >>> assert id_.startswith(id)

There's no output from the server.

    >>> server.print_output()

Stop the server.

    >>> server.stop()
    -15
