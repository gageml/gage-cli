# `gage log list` command

    >>> run("gage log list --help")  # +diff
    Show avaliable logs
    ⤶
    Usage: gage log list [OPTIONS]
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -m, --more...            Show more entries
      -l, --limit <N>          Limit list to N matching entries
      -a, --all                Show all matching entries
      -d, --deleted            Display deleted logs
      -h, --help               Print help

## Example

Generate some runs.

    >>> cd(make_temp_dir())

    >>> write_file("tasks.py", """
    ... from inspect_ai import task, Task
    ... from gage_inspect.dataset import dataset
    ...
    ... @task
    ... def foo():
    ...     return Task(model="mockllm/model")
    ...
    ... @task
    ... def bar():
    ...     return Task(model="mockllm/model")
    ...
    ... @dataset(task="bar")
    ... def bar_samples():
    ...     return ["a", "b", "c"]
    ... """)

    >>> for i in range(4):
    ...     task = "foo" if i % 2 else "bar"
    ...     run(f"gage run {task} -i '' -y", quiet=True)

List runs.

    >>> run("gage log list", cols=74)  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬───────────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model         │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼───────────────┼─────────┼─────────┤
    │ 1 │ {id1}  │ foo  │ run  │ success │ mockllm/model │         │ now     │
    │ 2 │ {id2}  │ bar  │ run  │ success │ mockllm/model │         │ now     │
    │ 3 │ {id3}  │ foo  │ run  │ success │ mockllm/model │         │ now     │
    │ 4 │ {id4}  │ bar  │ run  │ success │ mockllm/model │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴───────────────┴─────────┴─────────╯

Verify log Ids.

    >>> for name in sorted(ls_list("logs"), reverse=True):  # +parse
    ...     print(name)
    {}_foo_{id1_}.eval
    {}_bar_{id2_}.eval
    {}_foo_{id3_}.eval
    {}_bar_{id4_}.eval

    >>> assert id1_.startswith(id1)
    >>> assert id2_.startswith(id2)
    >>> assert id3_.startswith(id3)
    >>> assert id4_.startswith(id4)

## Limit

Limit to 2 entries.

    >>> run("gage log list -l 2")  # +parse +table
    ╭───┬────────┬──────┬──────┬─────────┬─────────┬─────────┬─────────╮
    │ # │ Id     │ Task │ Type │ Status  │ Model   │ Dataset │ Started │
    ├───┼────────┼──────┼──────┼─────────┼─────────┼─────────┼─────────┤
    │ 1 │ {id1_} │ foo  │ run  │ success │ mockll… │         │ now     │
    │ 2 │ {id2_} │ bar  │ run  │ success │ mockll… │         │ now     │
    ╰───┴────────┴──────┴──────┴─────────┴─────────┴─────────┴─────────╯
    Showing 2 of 4 (-m for more)

    >>> assert id1_ == id1
    >>> assert id2_ == id2

# Invalid options

`--limit` with `--more`.

    >>> run("gage log list -l 2 -m")
    --more and --limit cannot both be used
    <1>

`--limit` with `--all`.

    >>> run("gage log list -l 2 -a")
    --all and --limit cannot both be used
    <1>
