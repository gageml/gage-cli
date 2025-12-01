---
test-options: +skip=!GAGE_INSPECT
---

# `add` example

    >>> cd(os.environ["GAGE_INSPECT"], "examples", "add")

    >>> run("gage task list")
    ╭──────┬──────────────────┬────────╮
    │ Task │ Description      │ Source │
    ├──────┼──────────────────┼────────┤
    │ add  │ Add two numbers. │ add.py │
    ╰──────┴──────────────────┴────────╯

    >>> run("gage run add --input '{x: 1, y: 2}' -m mockllm/model -y")
    ┌  Run task
    │
    ◇  Task:
    │  add
    │
    ├  Description:
    │  Add two numbers.
    │
    ◇  Input:
    │  {x: 1, y: 2}
    │
    ◇  Target:
    │  None
    │
    ◇  Model:
    │  mockllm/model
    │
    ●  Output:
    │
    │  Default output from mockllm/model
    │
    └  Done

    >>> run("gage eval -m mockllm/model -y")  # +wildcard
    ┌  Evaluate tasks
    │
    ◇  Tasks:
    │  add
    │
    ◇  Model:
    │  mockllm/model
    │
    ╭──────────────────────────────────────────────────────────────────────────────╮
    │add (100 samples): mockllm/model                                              │
    ╰──────────────────────────────────────────────────────────────────────────────╯
    max_tasks: 4, tags: type:eval, dataset: add_tests
    ⤶
    total time:                                   0:00:...
    ⤶
    match
    accuracy  0.000
    stderr    0.000
    ⤶
    Log: ...
