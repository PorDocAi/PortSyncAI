# Task 4V work/document API evidence

- Extended work-assignment coverage for legacy and sidecar v2 MSDS review gates.
- v2-only cargo (`cargo_document_id = NULL`) accepts a confirmed MSDS and rejects pending review with 422 `MSDS_REVIEW_NOT_CONFIRMED`.
- Multiple MSDS mappings require every mapped review to be confirmed; pending/rejected mappings return 422 `MSDS_REVIEW_NOT_CONFIRMED`.
- Confirmed v2 MSDS takes precedence over a pending legacy document and returns 201.
- PATCH revalidates the existing cargo when `cargo_item_id` is omitted.
- The existing handler helper passed all cases; no handler fix was required.

## Verification

- `cargo test -p api --test work_assignment` — 12 passed, 0 failed.
- Tests use the in-process router and database harness; no manual QA required.
- Rust LSP diagnostics were unavailable because `rust-analyzer` is not installed.
