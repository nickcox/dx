## 1. Terminal Geometry Propagation

- [x] 1.1 Extend `SpaceReservation` to retain the applied scroll count alongside the post-scroll command-line row.
- [x] 1.2 Introduce terminal-presentation metadata for interactive `MenuResult` selection and cancellation while preserving the clean single-candidate result.
- [x] 1.3 Extend menu action serialization so dirty replace and cancel payloads include integer `redrawRow` and `scrollRows` fields.
- [x] 1.4 Preserve geometry through shell offset conversion and all interactive action construction paths without adding it to clean or noop outcomes.

## 2. PowerShell Prompt Reconciliation

- [x] 2.1 Add a generated PowerShell helper that captures a safe pre-menu redraw origin using RawUI display-cell geometry and available PSReadLine prompt/continuation metadata.
- [x] 2.2 Validate returned dirty geometry against the pre-menu cursor, viewport bounds, and applied scroll amount before mutating the PowerShell buffer.
- [x] 2.3 Redraw valid dirty replacements and cancellations with `PSConsoleReadLine::InvokePrompt($null, <targetY>)`, preserving replacement and cancellation semantics.
- [x] 2.4 Add safe behavior for unsupported explicit-Y invocation and invalid cancellation geometry without triggering native completion after explicit cancel.
- [x] 2.5 Determine through terminal testing whether explicit redraw needs a target-line clear and, if required, add only the narrow clear needed to prevent stale glyphs.

## 3. Automated Verification

- [x] 3.1 Add Rust unit tests for reservation geometry, dirty result propagation, replace/cancel JSON shapes, and clean/noop geometry omission.
- [x] 3.2 Add generated-hook tests for PowerShell geometry validation, explicit-Y `InvokePrompt`, invalid-geometry fallback, and unchanged clean replacement behavior.
- [x] 3.3 Add tests for redraw-origin calculations covering one-line, wrapped, explicit multiline, continuation-prompt, wide-character, and cursor-in-middle buffers.
- [x] 3.4 Confirm Bash, Zsh, and Fish continue accepting dirty action payloads with additional geometry fields and preserve existing repaint behavior.

## 4. Interactive And Final Verification

- [x] 4.1 Exercise PowerShell selection and Escape cancellation near the terminal bottom with one-line, multiline-prompt, wrapped-input, explicit-multiline, cursor-in-middle, and prediction-enabled cases.
- [x] 4.2 Verify behavior in the available native terminal and at least one terminal multiplexer or second emulator, recording any PSReadLine-version limitations in the design.
- [x] 4.3 Run focused menu and shell-hook tests, the complete Rust test suite, formatting, lint checks, and strict OpenSpec validation.
