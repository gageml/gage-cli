---
test-options: +skip=!GAGE_INSPECT
---

# `add` example

    >>> cd(os.environ["GAGE_INSPECT"], "examples", "quick-start")

    >>> run("gage task list")
    ╭───────┬──────────────────────┬──────────╮
    │ Task  │ Description          │ Source   │
    ├───────┼──────────────────────┼──────────┤
    │ funny │ Quick start example. │ funny.py │
    ╰───────┴──────────────────────┴──────────╯

    >>> run("python funny.py cats mockllm/model")
    Default output from mockllm/model

    >>> run("gage run funny -i cats -m mockllm/model -y")
    ┌  Run task
    │
    ◇  Task:
    │  funny
    │
    ├  Description:
    │  Quick start example.
    │
    ◇  Input:
    │  cats
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

    >>> run("gage eval funny -m mockllm/model -y")  # +wildcard
    ┌  Evaluate tasks
    │
    ◇  Tasks:
    │  funny
    │
    ◇  Model:
    │  mockllm/model
    │
    ╭──────────────────────────────────────────────────────────────────────────────╮
    │funny (5 samples): mockllm/model                                              │
    ╰──────────────────────────────────────────────────────────────────────────────╯
    max_tasks: 4, tags: type:eval, dataset: samples
    ⤶
    total time:                                   0:00:...
    ⤶
    llm_judge
    accuracy   0.000
    stderr     0.000
    ⤶
    Log: ...
