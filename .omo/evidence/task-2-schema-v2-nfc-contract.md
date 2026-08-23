# Task 2 evidence: schema v2 model-source contract

## Scope and baseline

- Worktree: `/Users/wingwogus/Projects/PortDocAI-schema-v2-nfc-contract`
- Branch: `refactor/#29-schema-v2-nfc-contract`
- Starting HEAD: `320624462e9416648b056820f8ea4ef77f7d6ecc`
- PR #48 head ancestry check: `git merge-base --is-ancestor 320624462e9416648b056820f8ea4ef77f7d6ecc HEAD` exited `0`.
- Initial task-worktree status was clean. The original worktree was not edited.
- Source-model style was established from existing `apis/api/models/*.json`: one VesperTide model per JSON file, inline keys/foreign keys/indexes, named composite unique groups, snake_case names, and Korean domain comments.

## Failing-first contract check

The narrow checker was added before model-source changes and run against the 25-model PR #48 baseline:

`python3 .omo/evidence/task-2-validate-schema-v2.py`

Exit: `1` (expected failure).

```text
schema-v2 contract: FAIL (36 errors)
- missing model: cargo_item_documents
- missing model: container_cargo_items
- missing model: containers
- missing model: education_completions
- missing model: education_courses
- missing model: education_target_rules
- missing model: equipment_check_events
- missing model: equipment_tag_tokens
- missing model: gate_events
- missing model: gate_terminals
- missing model: ppe_requirements
- missing model: shared_equipment_claims
- missing model: work_assignment_equipment
- missing model: work_ppe_requirement_snapshots
- missing model: work_preparations
- missing model: work_stops
- missing model: work_targets
- missing model: work_types
- missing model: works
- cargo_documents: missing column document_version
- cargo_documents: missing column processing_status
- cargo_documents: missing column reviewed_at
- cargo_documents: missing column reviewed_by_id
- work_assignments: missing column selected_at
- work_assignments: missing column status
- work_assignments: missing column work_assignment_id
- work_assignments: missing column work_id
- equipment: missing column manufacturer_replacement_due_at
- equipment: missing column owner_employee_id
- equipment: missing column ownership_type
- equipment: missing column status
- cargo_documents.document_type: missing enum values ['CI', 'MSDS']
- work_assignments: unique uq_work_assignment_work_employee must cover ['employee_id', 'work_id']
- equipment: legacy nfc_tag_uid must not remain in the v2 source
- equipment_check_logs: legacy day-wide equipment unique must be removed
- missing conversion map: .omo/evidence/task-2-pr48-conversion-map.md
```

This demonstrated missing #29/#34 tables and fields, the absent `UNIQUE(work_id,employee_id)`, raw `nfc_tag_uid`, day-wide equipment uniqueness, and the absent conversion map.

## Implemented contract

The source inventory now covers containers/mixed cargo/document roles, expanded cargo-document processing and review, work topology, assignment selection, education, reviewed PPE requirements and immutable work snapshots, equipment ownership/lifecycle and separately hashed tag tokens, historical allocations, portable shared claims, append-only equipment-check/idempotency events, assignment preparation, work stops, preserved terminal credentials, and append-only gate events.

The explicit PR #48 source/target/conversion/drop policy is in `.omo/evidence/task-2-pr48-conversion-map.md`. Its safe default is fail-closed: legacy assets become `SHARED`/ownerless/`BLOCKED`/tokenless; unprovable attendance-to-assignment links remain null with legacy identities preserved; no opaque token, assignment, or readiness fact is invented.

## Automated validation

Commands:

```sh
jq empty apis/api/models/*.json
python3 .omo/evidence/task-2-validate-schema-v2.py
```

Results:

- `jq empty`: PASS for all `44` source JSON files.
- Contract checker:

```text
schema-v2 contract: PASS
validated 22 required #29/#34 source models
validated work assignment, token hash, idempotency, claim, and gate-event identities
validated PR #48 conversion map anchors
```

The checker parses every model and asserts required #29/#34 names/fields/enums, `UNIQUE(work_id,employee_id)`, unique token hash shape, worker-scoped idempotency, historical allocation uniqueness, portable shared-claim uniqueness, append-only gate-event identity, absence of raw equipment UID/day-wide unique, and conversion-map anchors.

## Literal jq inventory and manual QA artifact

Command:

```sh
jq -r '
  def cols: [.columns[].name];
  def uniques: [.columns[] | select(.unique != null) | {column: .name, unique: .unique}];
  select(.name == "work_assignments" or .name == "equipment" or .name == "equipment_tag_tokens" or .name == "equipment_check_events" or .name == "shared_equipment_claims" or .name == "gate_events") |
  {table: .name, columns: cols, unique_columns: uniques}
' apis/api/models/*.json
```

Output:

```json
{
  "table": "equipment_check_events",
  "columns": [
    "equipment_check_event_id",
    "employee_id",
    "work_assignment_id",
    "equipment_id",
    "idempotency_key",
    "request_fingerprint",
    "http_status",
    "accepted",
    "reason_code",
    "response_json",
    "client_scanned_at",
    "occurred_at"
  ],
  "unique_columns": [
    {
      "column": "employee_id",
      "unique": "uq_equipment_check_employee_idempotency"
    },
    {
      "column": "idempotency_key",
      "unique": "uq_equipment_check_employee_idempotency"
    }
  ]
}
{
  "table": "equipment_tag_tokens",
  "columns": [
    "equipment_tag_token_id",
    "equipment_id",
    "tag_token_hash",
    "is_active",
    "issued_by_id",
    "issued_at",
    "deactivated_by_id",
    "deactivated_at",
    "created_at"
  ],
  "unique_columns": [
    {
      "column": "tag_token_hash",
      "unique": true
    }
  ]
}
{
  "table": "equipment",
  "columns": [
    "equipment_id",
    "equipment_type_id",
    "asset_number",
    "ownership_type",
    "owner_employee_id",
    "status",
    "status_reason",
    "braille_label",
    "manufacturer_replacement_due_at",
    "created_at",
    "updated_at"
  ],
  "unique_columns": [
    {
      "column": "asset_number",
      "unique": true
    }
  ]
}
{
  "table": "gate_events",
  "columns": [
    "gate_event_id",
    "terminal_id",
    "gate_id",
    "work_assignment_id",
    "employee_id",
    "idempotency_key",
    "request_fingerprint",
    "decision",
    "reason_code",
    "reason_codes",
    "decision_input_versions",
    "http_status",
    "response_json",
    "occurred_at",
    "legacy_verify_log_id",
    "legacy_attendance_id",
    "legacy_work_date"
  ],
  "unique_columns": [
    {
      "column": "terminal_id",
      "unique": "uq_gate_event_terminal_idempotency"
    },
    {
      "column": "idempotency_key",
      "unique": "uq_gate_event_terminal_idempotency"
    },
    {
      "column": "legacy_verify_log_id",
      "unique": true
    }
  ]
}
{
  "table": "shared_equipment_claims",
  "columns": [
    "shared_equipment_claim_id",
    "work_assignment_id",
    "equipment_id",
    "claimed_at"
  ],
  "unique_columns": [
    {
      "column": "work_assignment_id",
      "unique": "uq_shared_equipment_assignment"
    },
    {
      "column": "equipment_id",
      "unique": [
        "uq_shared_equipment_assignment",
        "uq_shared_equipment_active_claim"
      ]
    }
  ]
}
{
  "table": "work_assignments",
  "columns": [
    "work_assignment_id",
    "work_id",
    "employee_id",
    "assigned_by_id",
    "status",
    "selected_at",
    "started_at",
    "completed_at",
    "created_at",
    "updated_at"
  ],
  "unique_columns": [
    {
      "column": "work_id",
      "unique": "uq_work_assignment_work_employee"
    },
    {
      "column": "employee_id",
      "unique": "uq_work_assignment_work_employee"
    }
  ]
}
```

A Python assertion pass over the same parsed JSON confirmed the named composite keys, 64-character unique hash, shared-claim lock, primary gate-event identity, and absence of `gate_events.updated_at`: `manual assertions: PASS`.

## Adversarial and scope checks

- Malformed input: `jq empty apis/api/models/*.json` parsed every model.
- Dirty worktree: worktree was clean before task edits; only model and task evidence paths are eligible for staging.
- Stale state: branch/HEAD and PR #48 ancestry were captured above before edits.
- Historical migrations: no changes to `apis/api/migrations/0001*` through `0004*`.
- Generated entities/routes: no `apis/api/src/models/*.rs` or route changes; no entity export was run.
- Cleanup: no services, containers, databases, or background processes were created; none require cleanup.
