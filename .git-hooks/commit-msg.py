import re
import sys


def error(msg):
    raise SystemExit(f"Invalid commit message: {msg}")


def check(lines):
    # Strip comments
    lines = [line for line in lines if not line[:1] == "#"]

    if not lines:
        raise error("message cannot be empty")

    title = lines[0]

    # Title starts with [A-Z]
    if not re.match("[A-Z].*", title):
        error("title must start with a capital letter")

    # Title can't end with tab or space or punctuation
    if re.match(r".*[^0-9a-zA-Z`\"']$", title):
        error("title has trailing whitespace or punctuation")

    # Title <= 50 chars
    if len(title) > 50:
        error("title exceeds 50 chars")

    # Okay to not have detail
    if len(lines) == 1:
        return

    # One blank line after title
    if lines[1] != "":
        error("line 2 must be blank")

    # Not uncommon to leave a single training black - is okay
    if len(lines) == 2:
        return

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
        ([""], "title must start with a capital letter"),
        (["123"], "title must start with a capital letter"),
        (["A "], "title has trailing whitespace or punctuation"),
        (["A."], "title has trailing whitespace or punctuation"),
        (["A,"], "title has trailing whitespace or punctuation"),
        (["`thing`"], "title must start with a capital letter"),
        (["A" * 51], "title exceeds 50 chars"),
        (["A", "B"], "line 2 must be blank"),
        (["A", "", ""], "line 3 cannot be blank"),
        (["A", "", "A "], "line 3 has trailing whitespace"),
        (["A", "", "A" * 73], "line 3 is longer than 72 chars"),
    ]

    for lines, msg in error_cases:
        with raises(SystemExit) as e:
            try:
                check(lines)
            except SystemExit:
                raise
            else:
                raise AssertionError(f"did not raise SystemExit: {lines}")
        assert e.value.args[0] == f"Invalid commit message: {msg}", lines

    check(["This is okay"])
    check(["This is okay", "", "As is this"])
    check(["This is 'ok'"])
    check(["This is `ok`"])
    check(['This is "ok"'])
    check(["A" * 50])
    check(["A" * 50, "", "B" * 72])
    check(["Okay", "#"])
    check(["Okay", "# " + "A" * 72])
    check(["Okay", ""])  # Single trailing empty line okay


if __name__ == "__main__":
    msg_filename = sys.argv[1].strip()
    check([re.sub(r"(\n|\r\n)$", "", line) for line in open(msg_filename).readlines()])
