# `gage log delete` command

    >>> run("gage log delete --help")  # +diff
    Delete logs
    ⤶
    Usage: gage log delete [OPTIONS] [LOG]...
    ⤶
    Arguments:
      [LOG]...
              The target logs to delete
    ⤶
              LOGS may be specified using their # or Id. To delete a
              range, use one of 'START:', ':END', or 'START:END'.
    ⤶
    Options:
          --log-dir <LOG_DIR>
              Log directory
    ⤶
      -a, --all
              Delete all logs
    ⤶
      -y, --yes
              Don't prompt for dialog
    ⤶
      -p, --permanent
              Permanently delete logs
    ⤶
      -h, --help
              Print help (see a summary with '-h')
