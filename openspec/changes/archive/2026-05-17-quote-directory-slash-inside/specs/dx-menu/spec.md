## MODIFIED Requirements

### Requirement: Selection Replacement Semantics
For `replace` actions, `replaceStart` and `replaceEnd` SHALL define a half-open byte range in the original buffer to replace. `value` SHALL be the formatted replacement token produced for the selected candidate.

For `paths` mode, the replacement formatter SHALL preserve the user's query style when practical:

- cwd-relative selections MAY be emitted as `./child/`
- parent-relative selections MAY be emitted as `../sibling/`
- explicitly absolute input SHALL preserve absolute replacement output

Paths-mode replacements SHALL include a trailing slash and MAY include shell quoting when needed for the selected path text.

When a replacement value needs both shell quoting and an appended trailing directory slash, the trailing slash SHALL be included inside the quoted path token.

The parser SHALL continue to accept previously emitted replacements that placed an appended trailing directory slash outside the quoted path token.

For non-`paths` modes, replacements SHALL identify the selected destination without a trailing slash and MAY include shell quoting when needed.

Replacement bounds SHALL only target the active query token under the cursor and SHALL NOT modify unrelated buffer segments.

#### Scenario: Relative query preserves dot-slash style
- **WHEN** buffer is `cd b`, cwd contains `./benches`, and the selected candidate is that child directory
- **THEN** the returned replacement value MAY be `./benches/`

#### Scenario: Explicit absolute query preserves absolute replacement
- **WHEN** buffer is `cd /tmp/b`, the selected candidate is `/tmp/benches`, and `paths` mode is active
- **THEN** the returned replacement value SHALL be `/tmp/benches/`

#### Scenario: Quoted directory replacement keeps slash inside quotes
- **WHEN** the selected directory replacement requires shell quoting
- **THEN** the returned replacement value SHALL include the trailing `/` inside the quoted path token

#### Scenario: Previously emitted outside-slash quoted replacement remains parseable
- **WHEN** the shell buffer contains a previously emitted quoted replacement with the trailing `/` outside the quoted path token
- **THEN** `dx menu` SHALL parse it as the same raw query with the trailing slash preserved

#### Scenario: Replace only query token
- **WHEN** buffer is `cd pr --flag` and the selected replacement token is `./projects/`
- **THEN** replacement bounds SHALL cover only `pr`, and resulting buffer SHALL be `cd ./projects/ --flag`

#### Scenario: Preserve command prefix
- **WHEN** buffer is `up co` and user selects `/home/user/code`
- **THEN** replacement SHALL preserve `up ` prefix and update only selector token
