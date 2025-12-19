import fnmatch
import os
from queue import Empty, Queue
import re
import shlex
import shutil
import subprocess
import tempfile
from threading import Thread
import time
from typing import Any, cast

from groktest import parse_type


@parse_type("id", r"[a-zA-Z0-9]{22}")
def parse_id(s: str):
    return s


@parse_type("isodate", r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}[+\-]\d{2}:\d{2}")
def parse_isodate(s: str):
    return s


def _format_tz(s: str):
    # Add ':' to tz component for parsing by `fromisoformat`
    tz = s[19:]
    if not tz or tz[0] == ".":  # dot -> tz is actually a decimal part
        return s
    if len(tz) == 5:
        return s[:19] + tz[:3] + ":" + tz[3:]
    else:
        assert len(tz) == 7, s
        return s[:19] + tz[:3] + ":" + tz[3:5] + ":" + tz[5:]


def option_table(val: Any):
    if True:
        return normalize_table


def run(
    cmd: str,
    env: dict[str, str] | None = None,
    cwd: str | None = None,
    capture: bool = False,
    delenv: list[str] | None = None,
    strip_ansi=True,
    strip_trailing_spaces=True,
    quiet: bool = False,
    timeout: float = 10.0,
    cols: int = 68,
):
    if cols:
        if env:
            env["COLUMNS"] = str(cols)
        else:
            env = {"COLUMNS": str(cols)}
    p = _proc(cmd, cwd, env, delenv)
    out, err = p.communicate(timeout=timeout)
    assert err is None
    out = out.strip().decode()
    if strip_ansi:
        out = _strip_ansi(out)
    if strip_trailing_spaces:
        out = _strip_trailing_spaces(out)
    exit_code = cast(int, p.returncode)
    if capture:
        return exit_code, out
    if out and (not quiet or exit_code != 0):
        print(out)
    if exit_code != 0:
        print(f"<{exit_code}>")


def _proc(
    cmd: str,
    cwd: str | None = None,
    env: dict[str, str] | None = None,
    delenv: list[str] | None = None,
):
    split_cmd = shlex.split(cmd)
    os_env = _filter_env(dict(os.environ), delenv)
    env = {**os_env, **env} if env else os_env
    return subprocess.Popen(
        split_cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        env=env,
        cwd=cwd,
    )


def _filter_env(env: dict[str, str], delenv: list[str] | None):
    return {name: env[name] for name in env if name not in delenv} if delenv else env


_ansi_p = re.compile(r"\033\[[;?0-9]*[a-zA-Z]")


def _strip_ansi(s: str):
    return _ansi_p.sub("", s)


def _strip_trailing_spaces(s: str):
    return "\n".join([line.rstrip() for line in s.split("\n")])


def cd(*path: str):
    os.chdir(os.path.join(*path))


class Chdir:
    _save = None

    def __init__(self, path):
        self.path = path

    def __enter__(self):
        self._save = os.getcwd()
        os.chdir(self.path)

    def __exit__(self, *exc):
        assert self._save is not None
        os.chdir(self._save)


def make_temp_dir(prefix: str = "gage-test-"):
    return tempfile.mkdtemp(prefix=prefix)


def touch(filename: str):
    open(filename, "ab").close()
    now = time.time()
    os.utime(filename, (now, now))


def copy(src: str, dst: str):
    shutil.copyfile(src, dst)


def write_file(
    filename: str,
    contents: str,
    append: bool = False,
):
    opts = "a" if append else "w"
    with open(filename, opts) as f:
        f.write(contents)


def workspace_path(*names):
    workspace_dir = os.path.dirname(os.path.dirname(__file__))
    return os.path.join(workspace_dir, *names)


def _apply_project_paths():
    os.environ["PATH"] = os.path.pathsep.join(
        [
            (
                workspace_path("target", "release")
                if os.getenv("TEST_RELEASE") == "1"
                else workspace_path("target", "debug")
            ),
            os.environ["PATH"],
        ]
    )


def ls(
    root: str = ".",
    follow_links: bool = False,
    include_dirs: bool = False,
    ignore: str | list[str] | None = None,
):
    paths = ls_list(root, follow_links, include_dirs)
    if ignore:
        paths = filter_paths(paths, ignore)
    if not paths:
        print("<empty>")
    else:
        for path in paths:
            print(path)


def filter_paths(paths: list[str], ignore: str | list[str]):
    if isinstance(ignore, str):
        ignore = [ignore]
    return [
        path
        for path in paths
        if not any((fnmatch.fnmatch(path, pattern) for pattern in ignore))
    ]


def ls_list(
    root: str = ".",
    followlinks: bool = False,
    include_dirs: bool = False,
    unsorted: bool = False,
):
    if not os.path.exists(root):
        raise FileNotFoundError(root)

    paths: list[str] = []

    def relpath(path: str, name: str):
        return os.path.relpath(os.path.join(path, name), root)

    for path, dirs, files in os.walk(root, followlinks=followlinks):
        for name in dirs:
            if include_dirs or os.path.islink(os.path.join(path, name)):
                paths.append(relpath(path, name))
        for name in files:
            paths.append(relpath(path, name))
    return paths if unsorted else sorted(paths)


TABLE_SUBS = [
    (re.compile("─+┬"), "─┬"),
    (re.compile("─+╮"), "─╮"),
    (re.compile(" +│"), " │"),
    (re.compile("╰─+"), "╰─"),
    (re.compile("─+╯"), "─╯"),
    (re.compile("─+┼"), "─┼"),
    (re.compile("─+┤"), "─┤"),
]


def normalize_table(s: str):
    def apply_subs(line: str):
        for p, repl in TABLE_SUBS:
            line = p.sub(repl, line)
        return line

    return "\n".join([apply_subs(line) for line in s.split("\n")])


def cat(*parts: str):
    with open(os.path.join(*parts), "r") as f:
        s = f.read()
        if not s:
            print("<empty>")
        else:
            if s[-1:] == "\n":
                s = s[:-1]
            print(s)


def sleep(seconds: float):
    time.sleep(seconds)


class Server:
    def __init__(self, cmd: str, text: bool = True):
        self._p = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=text,
            shell=True,
            bufsize=0,
            close_fds=os.name == "posix",
        )
        assert self._p.stdout
        self._q = Queue()
        t = Thread(target=self._poll)
        t.daemon = True
        t.start()
        self._stopped = False

    def _poll(self):
        assert self._p.stdout
        while True:
            line = self._p.stdout.readline()
            if not line:
                break
            self._q.put(line)

    def readline(self):
        if self._stopped:
            raise RuntimeError("server is stopped")
        try:
            return self._q.get_nowait()
        except Empty:
            return None

    def readlines(self):
        return iter(self.readline, None)

    def print_output(self):
        print("".join(self.readlines()), end="")

    def stop(self, wait: float = 0.1):
        status = self._p.poll()
        if status is not None:
            return status
        self._p.terminate()
        time.sleep(wait)
        if self._p.poll() is None:
            self._p.kill()
            time.sleep(wait)
        self._stopped = True
        return self._p.poll()

    def __del__(self):
        self.stop()


# Add build target dir to path unless running in CI
if not os.getenv("CI"):
    _apply_project_paths()

__all__ = [
    "Chdir",
    "Server",
    "cat",
    "cd",
    "copy",
    "ls_list",
    "ls",
    "make_temp_dir",
    "os",
    "option_table",
    "parse_id",
    "parse_isodate",
    "run",
    "sleep",
    "time",
    "touch",
    "write_file",
]
