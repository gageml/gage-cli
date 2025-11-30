# `gage log restore` command

    >>> run("gage log restore --help")  # +diff
    Restore deleted logs
    ⤶
    Usage: gage log restore [OPTIONS] [LOG]...
    ⤶
    Arguments:
      [LOG]...  The target log(s) to restore
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -a, --all                Restore all deleted logs
      -y, --yes                Don't prompt for dialog
      -h, --help               Print help
