# Review TUI

## To do

General:

- [ ] "So what?" problem when looking at the Logs screen - we see logs
      but what does this mean? How do I use this? What's the point?

      Possible scenario:
      - At least add a Score column
      - Mark each log of interest
      - Press `c` to general a comparison report

- [ ] Look for use of `Effect::Dim` and replace with
      `theme::Style::caption()` for cases where it's a caption.

### Free flow notes

#### `Enter` on a panel

I want to press `Enter` to get into a log screen item (e.g. target).
This should let me immediately write some notes. Let's just call them
notes. So, I navigate to a panel and press `Enter` and start typing
notes. `Ctrl+Enter` should save/apply the notes.

`Enter` might bring up a general item dialog that supports notes, codes,
etc. Then there are shortcuts from the Sample screen like `n` for notes,
`c` for codes, etc. that go right to the thing.

#### Annotations

I think maybe showing annotations of panel content below them in fixed
height boxes that can be expanded as needed might be the way to handle
showing notes, codes/tags, etc.

#### Toggle overview display details

I think there are "display toggle" topics in play here:

- Show/hide annotations
- Show/hide transcript details

E.g. `t` might toggle transcript details and `a` might toggle
annotations.

#### Key map notes

Possible key actions:

| Key     | Action                       |
| ------- | ---------------------------- |
| `Enter` | Open panel details           |
| `n`     | Add a note                   |
| `c`     | Add a code                   |
| `t`     | Show/hide transcript details |
| `a`     | Show/hide annotations        |

#### New transcript view

Transcript details might be a separate screen - e.g.
`review/screens/log/transcript.rs`. So something like this:

| Key | View       |
| --- | ---------- |
| `1` | Overview   |
| `2` | Transcript |
| `3` | Raw (JSON) |

#### Filtering

I definitely want a sample filter. E.g. `f` followed by something.

| Key | View                |
| --- | ------------------- |
| `I` | Show only Incorrect |
| `C` | Show only Correct   |
| `s` | Score (dialog)      |
| `c` | Clear filters       |

`I` and `C` are shortcuts for `s` (sort dialog) + char.

What else here?

#### Moving across eval logs

I think a `Ctrl-Up` and `Ctrol-Down` could navigate to previous and next
eval logs.

How would filter apply here?

I think having the ability to "mark" logs for something could apply. If
logs were marked, `Ctrl-Up` and `Ctrl-Down` would limit navigation to
those marked logs. Otherwise they would move across the entire list.

#### Diffing/comparing runs/logs

I'm interested to know what changed between two runs of different scores
(e.g. 1.0 vs 0.25). I'd like to do this:

- Mark the two logs (click checkbox or press `Space`)
- Press `a` for an Action dialog
- Press `c` for Compare

#### Want to see the underlying sample data

I'd like to press a key (e.g. `d` for data, etc.) to see what the
underlying sample looks like. I'm immediately worried when I see "Input"
that I'm being led down a road. How did this so called "Input" get
there!

I would go further and want to be able to review the dataset
independently of an eval. In working with the HumanEval dataset, the
underlying data is weird and I need to see it up close for development.

I really do need to review every single sample, carefully. It's very
easy to want this stuff to tell me the answer and then to move on. These
early answers are filled with error. It really is critical to examine
the details.

It's surprisingly hard to get to this point with the tools.

#### Note on showing "transcript" details

Show the user input and output. That's easy.

- Input
- Output
- Score

Target goes below this as it's associated with scoring. That's a
separate concern.

In between input and output are "things that are run" that of great
interest. From what I've seen these are solver steps but I think they
could be other topics as well (e.g. maybe tool calls, sandbox execs,
etc.)

Try this:

- Show a line in between Input and Output that shows an expand icon
  (e.g. plus sign or right arrow)
- When collapsed, the line is a comma (or bar, etc.) delimited list of
  step names
- When clicked, the block expands with the list of steps

Something like this:

- Input
  - > Steps: structured_input_template, generate
- Output
- Score

When the Steps line is expanded:

- Input
  - structured_input_template
  - generate
- Output
- Score

The `structured_input_template` section should look at the state change
and infer that the user prompt text was set. It doesn't need to show a
diff - we don't really care about the change, just what was set.

```
-----------------------------------------------------------------
| structured_input_template                                     |
-----------------------------------------------------------------
| User prompt:                                                  |
|                                                               |
|   What is 10265 + -10?                                        |
|                                                               |
|   Do not explain the result. Do not apply formatting. Reply   |
|   only with the numeric result.                               |
-----------------------------------------------------------------
```

The `generate` section might look like this:

```
-----------------------------------------------------------------
| generate                                                      |
-----------------------------------------------------------------
| Assistant (gpt-4.1-nano-2025-04-14):                          |
|                                                               |
|   10255                                                       |
-----------------------------------------------------------------
```

I think this will work. Input is the start and output is the finish.
Very simple.

#### Score details

I wonder about showing more info up front with scores. I like to see
Correct/Incorrect. But this is also a trap. Showing only this term leads
me to believe it's true.

How can we encourage skepticism here?

I'd almost like to see a banner message saying:

> Scores can be wrong. They are a function of the model answer, the test
> target, and the scoring algorithm. If any of these is wrong, the score
> can be wrong. It's important to carefully verify that the test target
> and scoring algorithm are correct.

#### Toggle levels of verbosity

Consider this: let the user presss `v` some number of times. Each press
of `v` cycles through verbosity levels:

- Verbose
- Extra verbose
- Normal

For any current view, if you want more info, press `v`. If you want less
info, press `v`. Simple!

### Log screen

Sooner:

- [ ] Score metrics in task header
- [ ] How to show full transcript? Is this a different view or stuff we
      expand/show in the Sample view?

Later:

- [ ] "Sample N of M" not accurate when a subset of samples have been
      run (use the actual sample IDs somewhere along with another N of M
      display, progress bar, etc.)
- [ ] Open and Axial codes
  - What to call?
    - Open code, comment, note - (working: note)
    - Axial code, tag, label, code - (working: ??)
- [ ] Overview view (sample index)
  - Serves as a samples index/list but more importantly...
  - Capture eval/log level notes
  - Capture eval/log level TODO items (how are these different from
    notes - or are they notes with an "action item" designation?)
  - Capture eval/log level tags
- [ ] Progress bar or other status to show progress on annotations (will
      need a way to designate a sample as having been processed - and
      how do we differentiate progress per user/contributor?)

Bugs/issues:

- [ ] Arrows up/down change active panel but doesn't sync with scroll
      position

### Logs screen

- [ ] Need a score column
- [ ] Bug when Samples col is sorted and try to toggle (off by one error
      seems)
- [ ] Type column (Run, Eval, Serve?) - maybe this is nice to have
- [ ] Filter by sample count
- [ ] Keys should be clickable

  - [ ] Sort dialog
  - [ ] Filter dialog

- [ ] Filter by created (something like this)
  - 'H' within the last hour
  - 'D' Today
  - 'W' This week
  - 'M' This month
  - 'Y' This year

### Table component

- [ ] vim keys for nav: jk -> up down; hl -> left right

### Buttons

- [ ] Spacebar same as Enter (clicks/presses)
