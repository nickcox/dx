## Context

`dx menu` formats the selected candidate into the JSON `replace.value` that shell hooks insert into the active command line. Built-in `paths` mode and mapped `directory` mode currently append a trailing `/` so users can continue expanding inside the selected directory. Mapped `path` mode allows both files and directories, but it currently formats all selections without considering the selected path type.

## Goals / Non-Goals

**Goals:**

- Append `/` for mapped `path` mode only when the selected candidate is a directory.
- Keep mapped `path` mode file replacements unchanged.
- Preserve existing quoting behavior, including putting trailing directory slashes inside quoted path tokens.
- Preserve relative query-style rendering for mapped `path` mode.

**Non-Goals:**

- Changing candidate sourcing, filtering, or ranking.
- Changing shell hooks or JSON action shape.
- Adding configuration for trailing slash behavior.
- Appending slashes for stack, ancestors, frecents, recents, or mapped `file` mode.

## Decisions

1. Determine directory-ness at replacement formatting time.

Mapped `path` mode needs to distinguish selected directories from files. The existing `format_selected_path` helper only receives a string and mode, so implementation should either extend formatting to accept directory metadata or make `format_selected_path_for_query_style` decide whether a trailing slash is required from the selected `Path`.

Alternative considered: append `/` to all mapped `path` selections. That would break selected files for commands such as `cat`, `open`, or tools expecting a file path.

2. Keep slash placement consistent with paths/directory mode.

When a selected directory needs quoting, mapped `path` mode should use the same rule as built-in `paths` mode and mapped `directory` mode: the trailing slash is included inside the quoted path token, such as `'/path with spaces/'`.

3. Do not change completion/path candidate identity.

The selected `PathBuf` remains the candidate identity used by the menu. Only the emitted replacement string changes.

## Risks / Trade-offs

- Directory checks depend on filesystem state at selection time -> This is acceptable because candidate sourcing already depends on filesystem state and the behavior should reflect the selected path as it exists when the action is produced.
- Symlink behavior may follow `Path::is_dir()` semantics -> Use the existing project pattern for directory checks unless tests reveal a need for explicit symlink policy.
- Helper signature changes may touch several tests -> Keep the change localized to formatting helpers and update tests rather than changing menu action shape.
