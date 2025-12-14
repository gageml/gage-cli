# `dataset list` command

    >>> run("gage dataset list -h")
    List datasets
    ⤶
    Usage: gage dataset list [OPTIONS]
    ⤶
    Options:
      -p, --path <PATH>  Path to find datasets
      -h, --help         Print help

## Example

Create a module to register a dataset.

    >>> cd(make_temp_dir())

    >>> write_file("test.py", """
    ... from gage_inspect.dataset import dataset
    ...
    ... @dataset
    ... def foo():
    ...     "Test dataset."
    ...     pass
    ... """)

List datasets.

    >>> run("gage dataset list")
    ╭─────────┬───────────────┬─────────╮
    │ Dataset │ Description   │ Source  │
    ├─────────┼───────────────┼─────────┤
    │ foo     │ Test dataset. │ test.py │
    ╰─────────┴───────────────┴─────────╯
