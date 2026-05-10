## 1. Init-Time Mapping Schema and Validation

- [x] 1.1 Implement parser for `DX_MENU_COMMAND_MAPPINGS` with grammar `<command>=<mode>` and validation for `path|directory|file`.
- [x] 1.2 Wire mapping parsing into `dx init <shell> --menu` generation rather than `dx menu` runtime.
- [x] 1.3 Make invalid mapping entries fail init generation instead of emitting partial hook registrations.

## 2. Menu Mode Dispatch and Behavior

- [x] 2.1 Add explicit `dx menu --mode <mode>` handling for mapped external-command invocations.
- [x] 2.2 Add a file-aware filesystem candidate source for mapped modes so `path` can include files + directories and `file` can produce file candidates.
- [x] 2.3 Implement fixed `dx-smart` candidate behavior for mapped commands.
- [x] 2.4 Enforce v1 active-token-only replacement scope for mapped commands.

## 3. Shell Integration

- [x] 3.1 Update Bash `dx init --menu` output to register mapped commands that invoke `dx menu --mode <mode>`.
- [x] 3.2 Update Zsh, Fish, and PowerShell `dx init --menu` outputs to embed generated command→mode routing into their existing shared menu handlers with explicit mode arguments.
- [x] 3.3 Ensure mapped-command noop/error paths preserve existing native fallback behavior in all shells.
- [x] 3.4 Document and test that mapping changes require re-running `dx init` to regenerate hooks.

## 4. Verification and Docs

- [x] 4.1 Add parser and init-generation unit tests for valid mappings, invalid-entry failure, and fixed `dx-smart` behavior.
- [x] 4.2 Add integration tests for mapped command token-only replacement and file/directory mode filtering (`path`, `directory`, `file`).
- [x] 4.3 Add shell hook generation tests asserting Bash mapped registrations plus Zsh/Fish/PowerShell generated shared-handler routing under `--menu` with explicit `--mode` arguments.
