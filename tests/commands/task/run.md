# `gage task run` command

    >>> run("gage task run --help")  # +diff
    Run a task
    ⤶
    Usage: gage task run [OPTIONS] [TASK]
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
