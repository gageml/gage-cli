import re
import sys


def error(msg):
    raise SystemExit(f"Invalid commit message: {msg}")


def check(lines):
    if not lines:
        raise error("message cannot be empty")

    line1 = lines[0]

    # First line starts with [A-Z]
    if not re.match("[A-Z].*", line1):
        error("line 1 must start with a capital letter")

    # First line can't end with tab or space
    if re.match(r".*[ \t]$", line1):
        error("line 1 has trailing whitespace")

    # First line <= 50 chars
    if len(line1) > 50:
        error("line 1 exceeds 50 chars")

    # Okay to not have detail
    if len(lines) == 1:
        return

    # One blank line after title
    if lines[1] != "":
        error("line 2 must be blank")

    if lines[2].strip() == "":
        error("line 3 cannot be blank")

    # Remaining lines
    for i, line in enumerate(lines[2:]):
        line_num = i + 3
        # Can't end with tab or space
        if re.match(r".*[ \t]$", line):
            error(f"line {line_num} has trailing whitespace")

        # <= 72 chars
        if len(line) > 72:
            error(f"line {line_num} is longer than 72 chars")


def test():
    from pytest import raises

    error_cases = [
        ([], "message cannot be empty"),
        ([""], "line 1 must start with a capital letter"),
        (["123"], "line 1 must start with a capital letter"),
        (["A "], "line 1 has trailing whitespace"),
        (["`thing`"], "line 1 must start with a capital letter"),
        (["A" * 51], "line 1 exceeds 50 chars"),
        (["A", "B"], "line 2 must be blank"),
        (["A", "", ""], "line 3 cannot be blank"),
        (["A", "", "A "], "line 3 has trailing whitespace"),
        (["A", "", "A" * 73], "line 3 is longer than 72 chars"),
    ]

    for lines, msg in error_cases:
        with raises(SystemExit) as e:
            check(lines)
        assert e.value.args[0] == f"Invalid commit message: {msg}", lines

    check(["This is okay"])
    check(["This is okay", "", "As is this"])
    check(["A" * 50])
    check(["A" * 50, "", "B" * 72])


if __name__ == "__main__":
    msg_filename = sys.argv[1].strip()
    check([re.sub(r"(\n|\r\n)$", "", line) for line in open(msg_filename).readlines()])
