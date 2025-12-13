# Changes

## 0.2.2 (not released)

Fix:

- Apply env vars for Gage activated profile
- Handle missing LF chars in `.env` when activating/using profile

Other changes:

- Rename "Transcript" section in Review simplified to "Messages" (align
  with Inspect nomenclature)

## 0.2.1 (2025-12-10)

Enhancements:

- `--verbose` option for `status` command (provides additional
  environment details to help troubleshoot installs)

Fixes:

- Stop looking for default logs dir when we hit a known project dir

Other changes:

- Wheels for Linux and macOS for Python abis 3.10 - 3.14
- Moved example tests to `gage-inspect` project

## 0.2.0 (2025-11-30)

Initial release
