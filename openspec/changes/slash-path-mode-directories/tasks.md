## 1. Replacement Formatting

- [x] 1.1 Update menu replacement formatting so mapped `path` mode can detect whether the selected candidate is a directory.
- [x] 1.2 Append a trailing `/` for mapped `path` mode directory selections while preserving relative rendering and slash-inside-quotes behavior.
- [x] 1.3 Keep mapped `path` mode file selections unsuffixed and leave `file`, stack, ancestors, frecents, and recents behavior unchanged.

## 2. Tests And Validation

- [x] 2.1 Add tests for mapped `path` mode directory replacements, including relative cwd-descendant output.
- [x] 2.2 Add tests for mapped `path` mode file replacements to confirm no trailing slash is appended.
- [x] 2.3 Add or update tests for quoted path-mode directory replacements with the trailing slash inside quotes.
- [x] 2.4 Run the relevant menu tests and full test suite if feasible.
