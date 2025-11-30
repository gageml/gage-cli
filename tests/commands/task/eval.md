# `gage task eval` command

    >>> run("gage task eval --help")  # +diff
    Evaluate tasks
    ⤶
    Usage: gage task eval [OPTIONS] [TASK]...
    ⤶
    Arguments:
      [TASK]...  Tasks to evaluate
    ⤶
    Options:
      -m, --model <MODEL>      Task model (may be used more than once)
      -T <TASK_ARG>            Task argument NAME=VALUE (may use more than once)
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
