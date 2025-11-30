import os
import re
import shlex
import shutil
import subprocess
import tempfile
import time
from typing import cast


def run(
    cmd: str,
    cwd: str | None = None,
    env: dict[str, str] | None = None,
    capture: bool = False,
    delenv: list[str] | None = None,
    strip_ansi=True,
    strip_trailing_spaces=True,
    quiet: bool = False,
    timeout: float = 10.0,
):
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


def cd(dirname: str):
    os.chdir(dirname)


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
):
    paths = ls_list(root, follow_links, include_dirs)
    if not paths:
        print("<empty>")
    else:
        for path in paths:
            print(path)


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


_apply_project_paths()

__all__ = [
    "Chdir",
    "cd",
    "ls",
    "ls_list",
    "make_temp_dir",
    "os",
    "run",
    "touch",
    "copy",
    "write_file",
]
