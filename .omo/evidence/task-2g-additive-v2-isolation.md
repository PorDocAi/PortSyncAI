# Todo 2G - additive v2 isolation from legacy FK parents

## Bottom line

V2 workflow state now lives only in new additive tables. The immutable legacy physical tables `cargo_documents`, `cargo_items`, `equipment`, `equipment_check_logs`, `work_assignments`, `attendances`, `gate_terminals`, and `gate_verify_logs` match migrations `0001`-`0004` exactly, so VesperTide emits no parent alteration, temp-table rewrite, or `DROP TABLE` for SQLite or PostgreSQL.

Worktree: `/Users/wingwogus/Projects/PortDocAI-schema-v2-nfc-contract`
Starting HEAD: `a0bb528fe80782d8404bb51e1259936f21bb8bdc`
Historical migrations `0001`-`0004` were not modified. No migration, entity, route, OpenAPI, UI, or CI artifact was generated or changed.

## Failing-first contract

The Todo 2 checker was changed first to replay migrations `0001`-`0004` and compare complete model column definitions/constraints rather than checking a hand-picked subset. Against the pre-2G models it exited `1` with 63 errors, including:

```text
schema-v2 contract: FAIL (63 errors)
- missing model: cargo_document_versions
- missing model: equipment_profiles
- missing model: v2_work_assignments
- cargo_documents: v2 column must move to an additive table: document_version
- cargo_documents: v2 column must move to an additive table: processing_status
- equipment: v2 column must move to an additive table: ownership_type
- equipment: v2 column must move to an additive table: status
- work_assignments: v2 column must move to an additive table: v2_work_id
- work_assignments: v2 column must move to an additive table: status
- equipment_check_logs.equipment_id: definition differs from migrations 0001-0004
```

This catches the root cause before migration generation: any new/removed/modified field, index, unique, FK, default, comment, or constraint on an immutable table fails the source contract.

## Implemented physical boundary

- `cargo_documents` is historical-only. Expanded type, processing, review, and version state moved to `cargo_document_versions`, with nullable unique FK `legacy_cargo_document_id`.
- `cargo_items` is unchanged. Document roles now connect to `cargo_document_versions` through `cargo_item_documents`.
- `work_assignments` is historical-only. Work linkage and assignment lifecycle moved to `v2_work_assignments`, with nullable unique FK `legacy_assignment_id`. New preparation/stops/gate/equipment tables reference the v2 assignment identity.
- `equipment` and `equipment_check_logs` are unchanged, including historical `nfc_tag_uid`, `is_active`, and `uq_equipment_workdate`. Ownership/status/profile state moved to `equipment_profiles`, with nullable unique FK `legacy_equipment_id`; tokens, allocations, claims, and check events use `equipment_profile_id`.
- `attendances`, `gate_terminals`, and `gate_verify_logs` remain exact historical physical tables. V2 stops/gate events retain nullable legacy-ID FKs for provenance without altering those parents.
- PPE requirements/snapshots reference the v2 document-version identity rather than treating the legacy document row as the v2 state record.

The conversion policy and ownership of each field are recorded in `.omo/evidence/task-2-pr48-conversion-map.md` under the explicit terms `retained legacy`, `v2 authoritative`, and `immutable legacy physical tables`.

## JSON and checker verification

Commands:

```sh
jq empty apis/api/models/*.json
python3 .omo/evidence/task-2-validate-schema-v2.py
```

Result:

```text
schema-v2 contract: PASS
validated 8 immutable legacy tables against migrations 0001-0004
validated 29 required legacy/v2 source models
validated additive legacy-ID bridges and v2 authoritative relationships
validated token, allocation, claim, check-event, preparation, stop, and gate contracts
validated PR #48 conversion map anchors
```

`jq empty` exited 0 for every source model.

## Literal model-vs-historical QA

The checker reconstructs the historical schema by replaying all `create_table` and `add_column` actions in migration files `0001` through `0004`. For each immutable table it compares:

- exact column name set (missing and extra columns fail),
- full ordered column definition objects, including type, nullability, PK, FK action, index, unique, default, and comment,
- table constraints.

The passing result above proves exact definitions for:

```text
attendances
cargo_documents
cargo_items
equipment
equipment_check_logs
gate_terminals
gate_verify_logs
work_assignments
```

This is stronger than comparing the current models to an old Git snapshot: the committed migrations themselves are the physical authority.

## VesperTide SQL drop/rewrite scan

Commands, run from `apis/api` without writing a migration:

```sh
vespertide sql --backend sqlite > /tmp/task-2g-sqlite.sql
vespertide sql --backend postgres > /tmp/task-2g-postgres.sql
```

Both plans contain the same 21 `CreateTable` actions and no other action type:

```text
cargo_document_versions, containers, education_courses, equipment_profiles,
work_types, cargo_item_documents, ppe_requirements, container_cargo_items,
education_completions, equipment_tag_tokens, education_target_rules, works,
v2_work_assignments, work_ppe_requirement_snapshots, work_targets,
equipment_check_events, gate_events, shared_equipment_claims,
work_preparations, work_stops, work_assignment_equipment
```

Automated case-insensitive scan result:

```text
sqlite:
  CREATE TABLE count=21
  ADD COLUMN count=0
  all DROP TABLE count=0
  cargo_documents: drop=0 temp=0 alter=0
  cargo_items: drop=0 temp=0 alter=0
  equipment: drop=0 temp=0 alter=0
  equipment_check_logs: drop=0 temp=0 alter=0
  work_assignments: drop=0 temp=0 alter=0
  attendances: drop=0 temp=0 alter=0
  gate_terminals: drop=0 temp=0 alter=0
  gate_verify_logs: drop=0 temp=0 alter=0
postgres:
  CREATE TABLE count=21
  ADD COLUMN count=0
  all DROP TABLE count=0
  cargo_documents: drop=0 temp=0 alter=0
  cargo_items: drop=0 temp=0 alter=0
  equipment: drop=0 temp=0 alter=0
  equipment_check_logs: drop=0 temp=0 alter=0
  work_assignments: drop=0 temp=0 alter=0
  attendances: drop=0 temp=0 alter=0
  gate_terminals: drop=0 temp=0 alter=0
  gate_verify_logs: drop=0 temp=0 alter=0
legacy parent SQL scan: PASS
```

A separate scan for `DROP TABLE|DELETE FROM|_temp` across both files returned no matches. Therefore the SQLite plan cannot trigger the parent-drop FK behavior that previously nulled `cargo_items.cargo_document_id` or failed on `equipment_check_logs.equipment_id`.

## Adversarial and scope receipts

- **Data loss / FK rewrite:** checker rejects any immutable-parent difference; SQL scan proves no `ALTER`, temp table, `DROP TABLE`, or `DELETE FROM` on any listed parent.
- **Malformed input:** `jq empty apis/api/models/*.json` passed.
- **Stale baseline:** starting HEAD was exactly `a0bb528`; physical comparison reads committed migrations rather than copied prose.
- **Dirty worktree:** pre-existing untracked `.omo/evidence/task-2d-stale-evidence-cleanup.md` and `.omo/evidence/task-3e-data-preserving-migration.md` were preserved and excluded from staging.
- **Misleading generator success:** no revision was generated; both live backend SQL plans were inspected directly.
- **Interruption / leftovers:** SQL captures were written only under `/tmp`; no `0005`, entity export, route, OpenAPI, fixture, service, database, or background process was created.
- **Scope:** `git diff --name-only -- apis/api/migrations apis/api/src apps .github` returned no paths. `git diff --check` passed (with only Git's existing CRLF normalization warning for the historical cargo-document model).
