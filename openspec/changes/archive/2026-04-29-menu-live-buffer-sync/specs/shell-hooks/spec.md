## ADDED Requirements

### Requirement: Incremental Menu Action Consumption
Shell hooks integrating `dx menu` SHALL consume incremental menu actions during an active menu session and apply query-token replacements to the shell input buffer in real time.

Hooks SHALL apply each incremental replace action atomically using shell-native buffer APIs.

#### Scenario: Hook applies typed incremental replacement
- **WHEN** hook receives an incremental replace action for typed query input
- **THEN** it SHALL update shell input buffer range immediately to match the action payload

### Requirement: Selection-Only Events Must Not Trigger Buffer Writes
Hook adapters SHALL NOT mutate shell input buffer for selection-only navigation updates when no replace action is emitted.

#### Scenario: Navigation event without replace action
- **WHEN** menu emits selection-state update without replace payload
- **THEN** hook SHALL not modify shell buffer text

### Requirement: Protocol Compatibility and Safe Fallback
If hooks cannot parse or support the incremental action protocol (for example version mismatch or malformed payload), they SHALL gracefully fall back to native completion behavior without leaving terminal/input state corrupted.

#### Scenario: Unsupported incremental protocol version
- **WHEN** hook detects unsupported action protocol metadata
- **THEN** it SHALL stop menu handling and invoke native completion fallback

#### Scenario: Malformed incremental payload
- **WHEN** hook receives malformed JSON during menu session
- **THEN** it SHALL abort menu handling safely and restore normal shell completion behavior
