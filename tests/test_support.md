# Test support

## `normalize_table`

`normalize_table` removes repeating table fill chars `─` and ` ` to
allow matches pattern matching in tables.

    >>> from tests import normalize_table

Here's a table.

    >>> table = """
    ... ╭──────────────────────┬───────────╮
    ... │ gage version         │ 0.2.1     │
    ... │ gage_inspect version │ 0.2.1     │
    ... │ inspect_ai version   │ 0.3.153   │
    ... │ Python version       │ 3.12.3    │
    ... │ .env                 │           │
    ... │ Log dir              │ logs      │
    ... │ Config               │ gage.toml │
    ... │ Active profile       │           │
    ... ╰──────────────────────┴───────────╯
    ... """.strip()

A pattern match that changes the fill chars won't ever match.

    >>> print(table)  # +parse +fails
    ╭──────────────────────┬─────────╮
    │ gage version         │ 0.2.{}  │
    │ gage_inspect version │ 0.2.1   │
    │ inspect_ai version   │ 0.3.153 │
    │ Python version       │ 3.12.3  │
    │ .env                 │         │
    │ Log dir              │ logs    │
    │ Config               │ {}      │
    │ Active profile       │         │
    ╰──────────────────────┴─────────╯

Normalizing the table removes the repeating fill chars.

    >>> print(normalize_table(table))
    ╭─┬─╮
    │ gage version │ 0.2.1 │
    │ gage_inspect version │ 0.2.1 │
    │ inspect_ai version │ 0.3.153 │
    │ Python version │ 3.12.3 │
    │ .env │ │
    │ Log dir │ logs │
    │ Config │ gage.toml │
    │ Active profile │ │
    ╰─┴─╯

Applying the `table` option normalizes the values being compared.

    >>> print(table)  # +parse +table
    ╭──────────────────────┬─────────╮
    │ gage version         │ 0.2.{}  │
    │ gage_inspect version │ 0.2.1   │
    │ inspect_ai version   │ 0.3.153 │
    │ Python version       │ 3.12.3  │
    │ .env                 │         │
    │ Log dir              │ logs    │
    │ Config               │ {}      │
    │ Active profile       │         │
    ╰──────────────────────┴─────────╯

Example with inner row border.

    >>> table = """
    ... ╭─────────┬────────────────────────╮
    ... │ Log     │ 8GpboexKmmkLbuuqXtpDTQ │
    ... ├─────────┼────────────────────────┤
    ... │ Task    │ test                   │
    ... │ Started │ now                    │
    ... │ Status  │ success                │
    ... │ Dataset │                        │
    ... │ Samples │ 1                      │
    ... │ Model   │ mockllm/model          │
    ... ╰─────────┴────────────────────────╯
    ... """.strip()

    >>> print(normalize_table(table))
    ╭─┬─╮
    │ Log │ 8GpboexKmmkLbuuqXtpDTQ │
    ├─┼─┤
    │ Task │ test │
    │ Started │ now │
    │ Status │ success │
    │ Dataset │ │
    │ Samples │ 1 │
    │ Model │ mockllm/model │
    ╰─┴─╯

    >>> print(table)  # +parse +table
    ╭─────────┬───────────────────────╮
    │ Log     │ {log_id_}             │
    ├─────────┼───────────────────────┤
    │ Task    │ test                  │
    │ Started │ now                   │
    │ Status  │ success               │
    │ Dataset │                       │
    │ Samples │ 1                     │
    │ Model   │ mockllm/model         │
    ╰─────────┴───────────────────────╯

## Server

Use `Server` to start and read output from a process.

Create a program that prints a line of output and waits.

    >>> cd(make_temp_dir())

    >>> write_file("simple.py", """
    ... import time, sys
    ...
    ... sys.stdout.write("Hello!\\n")
    ... sys.stdout.flush()
    ... time.sleep(5)
    ... """)

Start a server.

    >>> server = Server("python simple.py")

Wait a moment and read output.

    >>> sleep(0.1)
    >>> server.readline()
    'Hello!\n'

If there's nothing to read, `readline()` returns `None`.

    >>> print(server.readline())
    None

Stop the server.

    >>> server.stop()
    -15

Reading from the server after it's stopped generates an error.

    >>> server.readline()
    Traceback (most recent call last):
    RuntimeError: server is stopped

## Parse types

Ids:

    >>> print("T6uPKw9kyNxK36wFVEn3zn")  # +parse
    {s:id}

    >>> assert s == "T6uPKw9kyNxK36wFVEn3zn"

    >>> print("WsgenY4zbeqJu3TrwRc6P8")  # +parse
    {:id}


    >>> print("T6uPKw9kyNxK36wFVEn3z")  # +parse +fails (too short)
    {:id}

Timestamps:

    >>> print("2025-12-19T18:50:16+00:00")  # +parse
    {s:isodate}

    >>> assert s == "2025-12-19T18:50:16+00:00"

    >>> print("2025-12-19T18:50:16-06:30")  # +parse
    {:isodate}

    >>> print("2025-12-19T18:50:16+00:xx")  # +parse +fails (bad time)
    {:isodate}
