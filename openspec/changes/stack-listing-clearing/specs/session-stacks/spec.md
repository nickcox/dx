## MODIFIED Requirements

### Requirement: Output Contract
All stack action subcommands (`dx stack push`, `dx stack undo`, `dx stack redo`) SHALL follow a consistent output contract:
- **On success**: Print exactly one absolute path to stdout, followed by a newline. Exit with code 0.
- **On failure**: Print nothing to stdout. Print a human-readable diagnostic to stderr. Exit with a non-zero code.

`dx stack --list` and `dx stack --clear` SHALL follow their own command-specific output contracts as defined by stack inspection and stack maintenance capabilities.

#### Scenario: Successful operation output
- **WHEN** `dx stack undo --session 100` succeeds and the restored path is `/home/user`
- **THEN** stdout SHALL contain exactly `/home/user\n` and the exit code SHALL be 0

#### Scenario: Failed operation output
- **WHEN** `dx stack undo --session 100` fails because the undo stack is empty
- **THEN** stdout SHALL be empty, stderr SHALL contain a diagnostic message, and the exit code SHALL be non-zero
