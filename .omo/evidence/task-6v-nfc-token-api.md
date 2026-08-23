# Task 6V: sidecar opaque NFC token API evidence

## Delivered contract

- `POST /equipment-checks` now accepts only `work_assignment_id`, `tag_token`, `client_scanned_at`, and `idempotency_key`.
- Worker identity comes from the signed JWT `sub`; the route does not read legacy `equipment.nfc_tag_uid` or attendance/check-log tables.
- Token lookup hashes the exact case-sensitive request value with SHA-256. Raw token text is returned only by admin issuance/rotation and is not put in event JSON.
- Admin issue, profile issue alias, revoke, profile revoke alias, and rotate handlers use `equipment_tag_tokens`; issuance uses 256 bits of UUID-v4 randomness and persists only the 64-character hash.
- Personal ownership, shared claims, all equipment lifecycle statuses, assignment/work lifecycle, PPE snapshot matching, requirement progress, stable reason codes, append-only events, exact replay, changed-payload conflict, and new-key duplicate behavior are covered.

## Automated verification

From `apis/api`:

```text
cargo fmt --check -- (touched route/test files)
cargo clippy --all-targets -- -D warnings
cargo test --test tagging --test concurrency
cargo test
```

Observed results:

- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --test tagging --test concurrency`: 11 passed, 0 failed.
- `cargo test`: all unit/integration/doc targets passed (43 integration tests, 0 failed).
- Rust LSP diagnostics were unavailable because `rust-analyzer` is not installed.
- OpenAPI files were restored after test/build generation and are intentionally outside this task's file boundary.

## Behavior matrix covered

| Case | Expected result | Test |
|---|---|---|
| PERSONAL owner | `201 TAG_ACCEPTED` | `personal_owner_is_accepted_and_non_owner_is_rejected` |
| PERSONAL non-owner | `403 PERSONAL_EQUIPMENT_OWNER_MISMATCH` | `personal_profile_rejects_a_different_worker_with_a_known_token` |
| SHARED active claim | `409 SHARED_EQUIPMENT_IN_USE` | `shared_equipment_conflicts_across_active_assignments` |
| STOPPED work | claim retained, conflict remains | `shared_claim_is_retained_while_stopped_and_released_after_completion` |
| terminal work | claim released, next assignment accepts | same lifecycle test |
| BLOCKED/DAMAGED/LOST/REPLACED | exact `EQUIPMENT_*` 422 code | `all_non_available_lifecycle_states_are_rejected_with_stable_codes` |
| unknown/deactivated token | `404 TAG_NOT_REGISTERED` | `unknown_and_deactivated_tokens_are_not_registered` |
| same key, same payload | exact original response replay | `same_key_replays_exact_response_and_changed_payload_is_conflict` |
| same key, changed payload | `409 IDEMPOTENCY_KEY_REUSED` | same test |
| new key, same assignment/equipment | `200 TAG_ALREADY_ACCEPTED` | `new_key_duplicate_returns_already_accepted_without_a_second_allocation` |
| concurrent shared requests | one 201, nine 409, one claim/allocation, ten events | `ten_concurrent_shared_tags_yield_one_acceptance_and_nine_conflicts` |
| concurrent same key | ten exact 201 replays, one event/allocation | `concurrent_same_key_requests_replay_one_stored_response` |
| token storage | hash only, 64 hex chars | `admin_issue_and_revoke_only_stores_a_hash` |

## Manual temporary API QA (redacted)

A temporary SQLite API was started on port `18085` with a temporary JWT secret and stopped after the curl run. The fixture used one active worker assignment, one confirmed PPE snapshot, one available shared profile, and an administrator-issued token. The raw token and JWT are intentionally omitted here.

```text
GET /health
HTTP 200
{"status":"ok"}

POST /equipment-checks   # first request; tag_token=<REDACTED>, JWT=<REDACTED>
HTTP 201
{"accepted":true,"reason_code":"TAG_ACCEPTED","progress":{"required":1,"satisfied":1,"complete":true},...}

POST /equipment-checks   # identical body/key replay
HTTP 201
{"accepted":true,"reason_code":"TAG_ACCEPTED","progress":{"required":1,"satisfied":1,"complete":true},...}

# administrator changed the profile lifecycle to DAMAGED; new idempotency key
POST /equipment-checks   # tag_token=<REDACTED>, JWT=<REDACTED>
HTTP 422
{"accepted":false,"reason_code":"EQUIPMENT_DAMAGED","progress":{"required":1,"satisfied":0,"complete":false},...}
```

The manual run verified the real TCP API surface; response bodies were captured only after redacting credentials in this evidence.
