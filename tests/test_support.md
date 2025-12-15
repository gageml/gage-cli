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
