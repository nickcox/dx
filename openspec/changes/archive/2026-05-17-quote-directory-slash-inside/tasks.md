## 1. Replacement Formatting

- [x] 1.1 Update directory replacement formatting so appended trailing slashes are included before shell quoting is applied.
- [x] 1.2 Preserve non-slashed replacement formatting for selections that do not need an appended directory slash.
- [x] 1.3 Ensure the active `slash-path-mode-directories` change follows the same quote placement rule for mapped `path` mode directories.

## 2. Parser Compatibility

- [x] 2.1 Extend quoted-token parsing to accept slash-inside-quote forms such as `'/Library/Application Support/'`.
- [x] 2.2 Preserve parsing compatibility for outside-slash forms such as `'/Library/Application Support'/`.

## 3. Tests And Validation

- [x] 3.1 Update replacement formatting tests to expect quoted directory slashes inside quotes.
- [x] 3.2 Add parser tests for both inside-slash and outside-slash quoted directory forms.
- [x] 3.3 Run the relevant menu parser/formatting tests and full test suite if feasible.
