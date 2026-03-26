# Product Requirements Document: cd-extras CLI (Cross-Shell)

## 1. Overview
`cd-extras` is currently a highly capable directory navigation module written entirely in PowerShell. It provides features like stack-based movement (undo/redo), frecency-based directory jumps, path abbreviation expansion, bookmarks, and auto-cd.

**Goal:** Rewrite the core logic of `cd-extras` as a compiled, cross-shell binary (e.g., in Rust or Go) with thin, per-shell hooks (for Bash, Zsh, Fish, and PowerShell).

**Why?**
- **Universal History:** Frecency (recently/frequently visited directories) and bookmarks can be shared seamlessly across subshells and different terminal environments.
- **Performance:** Complex path expansions and frecency calculations are currently blocking shell execution. A compiled binary will reduce input latency to near zero.
- **Improved Concurrency:** The current PowerShell implementation relies on complex background runspaces and Mutexes to sync an underlying CSV file. A compiled binary utilizing embedded SQLite or robust file locking will handle simultaneous writes elegantly.

## 2. Architecture
The architecture comprises two distinct layers:
1. **The Core CLI Binary:** The engine that manages state, path resolution, and algorithm logic. It never changes directories itself (since a child process cannot alter a parent's working directory).
2. **The Shell Hooks:** Thin wrapper functions loaded into the user's shell profile (`.bashrc`, `.zshrc`, `config.fish`, `$PROFILE`). The shell handles input, invokes the CLI binary to resolve paths, natively changes the directory based on the CLI's output, and reports success back to the CLI.

### 2.1 Component Interaction Diagram
1. User types `cd pr/cd` and hits standard `Enter`.
2. The Shell Hook intercepts `cd` and calls: `dx resolve "pr/cd"`
3. The CLI expands the abbreviation to `/Users/nick/projects/cd-extras` and prints it to `stdout`.
4. The Shell Hook reads `stdout` and executes the built-in: `builtin cd "/Users/nick/projects/cd-extras"`
5. Upon successful navigation, the Shell Hook calls: `dx add "/Users/nick/projects/cd-extras" --session $PID` to record the visit.

## 3. Core Features & Requirements

### 3.1 Directory Navigation & Resolution (`resolve` command)
- Supports traditional path traversal (`..`, `../..`, `~`).
- Supports step-up navigation (`up` or `..` aliases).
- Expands abbreviated paths (e.g., `cd p/c/d` -> `/Users/nick/projects/code/dir`).
- Integrates `CD_PATH` (if implemented) or fallback searchable directories.

### 3.2 Stack-Based Movement (Undo/Redo)
- Supports traversing backwards (`cd-`, `~`) and forwards (`cd+`, `~~`) through the session's directory history.
- **Constraint:** Since stack history is intrinsically tied to a specific terminal session/tab, the CLI must track individual stacks keyed by the shell's Process ID (`$PID` / `$$`).
- Temporary state storage (e.g., JSON files in `/tmp/cd-extras-sessions/<PID>.json`) should automatically clean up across reboots.

### 3.3 Frecency & Recent Locations (`add`, `recent`, `frecent`)
- `cdr` (Recent): Jump to previously visited directories ordered chronologically.
- `cdf` (Frecent): Jump to directories ordered by a combination of frequency and recency.
- **Constraint:** Uses a fast, persistent store (e.g., SQLite or binary file in `~/.local/share/cd-extras/`) with file-locking to handle concurrent writes from multiple shell sessions.

### 3.4 Bookmarks
- `mark` / `unmark`: Allow users to explicitly save persistent alias-like bookmarks to specific directories.

### 3.5 Auto-CD (Optional)
- Allow users to simply type a directory name without the `cd` prefix.
- Handled natively in Bash (`shopt -s autocd`) and Zsh (`setopt autocd`).
- If custom abbreviations (like `pr/cd`) need Auto-CD, the shell hooks must implement a `command_not_found_handler` that queries the CLI before throwing a standard target-not-found error.

### 3.6 Shell Completions
- The CLI must be able to generate its own completion definitions dynamically.
- `dx complete "<context>"` should return tab-completed candidates instantly for paths, frecent entries, and bookmarks.

## 4. Technical Specifications

### 4.1 CLI Interface (Proposed)
```text
dx <command> [args]

Commands:
  resolve <query>         Resolves an abbreviation or fuzzy string to an absolute path
  add <path>              Registers a path to the frecency DB and the current session stack
  undo                    Returns the target path for moving backward in the stack
  redo                    Returns the target path for moving forward in the stack
  frecent <query>         Returns the best matching frecent directory
  recent [query]          Returns the most recent directory matching the query
  bookmarks               Manages bookmarks (add, remove, list)
  complete <type> <word>  Generates completion options for the shell hook
  init <shell>            Outputs the shell hook code (e.g., bash, zsh, fish, pwsh)
```

### 4.2 Installation & Initialization
To install and hook the binary, users would add the following to their shell configuration:
- **PowerShell:** `Invoke-Expression (& dx init pwsh)`
- **Zsh:** `eval "$(dx init zsh)"`
- **Bash:** `eval "$(dx init bash)"`
- **Fish:** `dx init fish | source`

## 5. Migration Plan
1. **Phase 1: Binary Engine foundation** — Build the CLI binary, focusing purely on state management (SQLite/JSON), Frecency calculations, and Path expansion logic. Write unit tests independent of any shell.
2. **Phase 2: PowerShell Hook** — Replace `cd-extras.psm1` with a lightweight wrapper script that delegates all logic to the new CLI. Ensure parity with existing functionality.
3. **Phase 3: Cross-Shell Support** — Write the `init` templates for Zsh, Bash, and Fish. Add completions support for these shells.
4. **Phase 4: Release** — Deprecate the pure-PowerShell module in favor of the new binary+hook distribution model (likely via Homebrew, Cargo, etc.).
