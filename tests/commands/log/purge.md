# `gage log purge` command

    >>> run("gage log purge --help")  # +diff
    Purge deleted logs
    ⤶
    Usage: gage log purge [OPTIONS] [LOG]...
    ⤶
    Arguments:
      [LOG]...  The target logs to permanently delete
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -a, --all                Permanently clean all deleted logs
      -y, --yes                Don't prompt for dialog
      -h, --help               Print help
