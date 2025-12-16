# Main

    >>> run("gage --help")  # +diff
    Usage: gage [OPTIONS] <COMMAND>
    ⤶
    Commands:
      dataset  Manage datasets
      delete   Delete one or more logs
      eval     Run an evaluation
      list     List eval logs
      log      Manage logs
      profile  Manage profiles
      review   Review logs
      run      Run a task
      status   Show project status
      task     Manage tasks
      help     Print this message or the help of the given subcommand(s)
    ⤶
    Options:
      -c, --config <PATH>  Gage config file (defaults to gage.toml)
          --debug          Enable debug logging
      -h, --help           Print help
      -V, --version        Print version

## Aliaes

### `gage delete`

    >>> run("gage delete --help")  # +diff
    Delete one or more logs
    ⤶
    Usage: gage delete [OPTIONS] [SPEC]...
    ⤶
    Arguments:
      [SPEC]...
              Log(s) to delete
    ⤶
              SPEC may be a log ID, index position, or index position
              range in the form START:END.
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

### `gage eval`

    >>> run("gage eval --help")  # +diff
    Run an evaluation
    ⤶
    Usage: gage eval [OPTIONS] [TASK]...
    ⤶
    Arguments:
      [TASK]...  Tasks to evaluate
    ⤶
    Options:
      -m, --model <MODEL>      Task model (may be used more than once)
      -T <TASK_ARG>            Task argument NAME=VALUE (may use more
                               than once)
      -d, --dataset <NAME>     Evaluation dataset
      -l, --limit <N>          Limit the number of samples to evaluate
      -s, --sample <ID>        Task model (may be used more than once)
          --sandbox <TYPE>     Sandbox environment type
          --shuffle            Suffle sample order
      -e, --epochs <N>         Number of times to evaluate dataset
      -p, --path <PATH>        Path to find tasks
      -y, --yes                Don't prompt to for input
          --log-dir <LOG_DIR>  Log directory
      -h, --help               Print help

### `gage list`

    >>> run("gage list --help")  # +diff
    List eval logs
    ⤶
    Usage: gage list [OPTIONS]
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -m, --more...            Show more entries
      -l, --limit <N>          Limit list to N matching entries
      -a, --all                Show all matching entries
      -d, --deleted            Display deleted logs
      -h, --help               Print help

### `gage review`

    >>> run("gage review --help")  # +diff
    Review logs
    ⤶
    Usage: gage review [OPTIONS]
    ⤶
    Options:
          --log-dir <LOG_DIR>  Log directory
      -h, --help               Print help

### `gage run`

    >>> run("gage run --help")  # +diff
    Run a task
    ⤶
    Usage: gage run [OPTIONS] [TASK]
    ⤶
    Arguments:
      [TASK]
              Task to run
    ⤶
    Options:
      -i, --input <INPUT>
              Task input
    ⤶
      -T <TASK_ARG>
              Task argument NAME=VALUE (may use more than once)
    ⤶
      -m, --model <MODEL>
              Task model
    ⤶
              If not specified, environent variables GAGE_MODEL and
              INSPECT_EVAL_MODEL are used. Otherwise the model defined
              by the task itself is used.
    ⤶
          --target <TARGET>
              Expected output
    ⤶
          --score
              Score the result (implied when --target is specified)
    ⤶
      -t, --tag <TAG>
              Task tag (may use more than once)
    ⤶
      -p, --path <PATH>
              Path to find tasks
    ⤶
          --log-dir <LOG_DIR>
              Inspect log dir
    ⤶
      -y, --yes
              Don't prompt for input
    ⤶
              --input is required when this option is used.
    ⤶
      -h, --help
              Print help (see a summary with '-h')
