# Todo 3F — sidecar workflow v2 migration

Verdict: **pass**. Generated `0005` is 21 additive sidecar `CreateTable` actions. SeaORM entities exported. Fresh and populated SQLite/PostgreSQL upgrades preserved every required count and sentinel. Commit: `feat(db): add sidecar workflow v2 migration`.

Worktree: `/Users/wingwogus/Projects/PortDocAI-schema-v2-nfc-contract`
Branch: `refactor/#29-schema-v2-nfc-contract`
Starting HEAD: `4aa758e73fb687351211ea3d715ab4118494893c`
Historical migrations `0001`–`0004` were not modified. JSON source models, routes, OpenAPI, UI, CI, and the original source worktree were not modified.

## Pre-gate SQL (before revision)

From `apis/api`:

```sh
vespertide sql --backend sqlite
vespertide sql --backend postgres
```

Both pending plans contained the same 21 `CreateTable` actions and no other action type:

```text
cargo_document_versions, containers, education_courses, equipment_profiles,
work_types, cargo_item_documents, ppe_requirements, container_cargo_items,
education_completions, equipment_tag_tokens, education_target_rules, works,
v2_work_assignments, work_ppe_requirement_snapshots, work_targets,
equipment_check_events, gate_events, shared_equipment_claims,
work_preparations, work_stops, work_assignment_equipment
```

Case-insensitive scan:

| Backend | CREATE TABLE | ADD COLUMN | ALTER TABLE | DROP TABLE | DELETE FROM | `_temp` |
|---|---|---|---|---|---|---|
| SQLite | 21 | 0 | 0 | 0 | 0 | 0 |
| PostgreSQL | 21 | 0 | 0 | 0 | 0 | 0 |

Legacy parent SQL scan (`cargo_documents`, `cargo_items`, `equipment`, `equipment_check_logs`, `work_assignments`, `attendances`, `gate_terminals`, `gate_verify_logs`): zero `CREATE`/`ALTER`/`DROP`/`TEMP`/`DELETE` against those parents. The `equipment_*` sidecar names matched the `equipment` prefix in a naive regex; exact `"equipment"` CREATE/ALTER/DROP hits were 0.

## Generated 0005 (inspected before export)

```text
Created migration: migrations/0005_sidecar_workflow_v2.vespertide.json
  Version: 5
  Actions: 21
  Comment: sidecar workflow v2
  id: 95b7c900-9f42-48c7-9ede-4c90b812dee0
```

Action types: 21 `create_table`. Zero `add_column` / `modify_column_*` / `remove_constraint` / `drop_table` / `delete_column` / `delete_rows`. Zero actions on the eight immutable legacy parents.

`vespertide status` after generation: schema synchronized.

`vespertide log` v5 SQL (runtime statements, no wrapping BEGIN/COMMIT counted):

| Backend | CREATE TABLE | DROP TABLE | ALTER TABLE | DELETE FROM | `_temp` |
|---|---|---|---|---|---|
| SQLite v5 | 21 sidecar tables | 0 | 0 | 0 | 0 |
| PostgreSQL v5 | 21 sidecar tables | 0 | 0 | 0 | 0 |

Historical v1–v4 SQL still contains the committed SQLite rewrite of `cargo_documents`/`attendances` in `0004`. That is not 0005.

## SeaORM export

```sh
vespertide export --orm seaorm
```

Wrote the 21 sidecar entity files and regenerated `src/models/mod.rs`. Existing entity files gained sidecar relations (`equipment` → `equipment_profiles`, `cargo_documents` → `cargo_document_versions`, …). Legacy physical columns remain: `equipment.nfc_tag_uid`/`is_active`, `work_assignments.assignment_id`, `cargo_items.cargo_document_id`, `gate_verify_logs.*`.

## Fresh migrations

Runtime: SeaORM 2 + `vespertide::vespertide_migration!` 0.1.61 (same path as `apis/api/src/main.rs` / tests). Each migration applies inside one transaction.

SQLite empty file DB, `PRAGMA foreign_keys=ON`, apply `0001`–`0005`: **PASS**.
PostgreSQL `task_3f_fresh`, apply `0001`–`0005`: **PASS**.

Both backends:

- 21 sidecar tables present
- legacy tables present
- `equipment` columns still include `nfc_tag_uid`, `is_active`
- `work_assignments` still uses `assignment_id` (no `v2_work_id` / `status` on the parent)
- `cargo_items.cargo_document_id` retained
- `vespertide_version` rows 1–5, v5 id `95b7c900-9f42-48c7-9ede-4c90b812dee0`

## Populated upgrades (explicit IDs)

Seeded after `0001`–`0004`, then applied 0005 with the same runtime.

Before/after `COUNT(*)` on both SQLite and PostgreSQL:

| table | before | after |
|---|---|---|
| cargo_documents | 2 | 2 |
| equipment | 2 | 2 |
| equipment_check_logs | 2 | 2 |
| work_assignments | 3 | 3 |
| cargo_items | 2 | 2 |
| gate_verify_logs | 2 | 2 |

Sentinel SQL (identical before and after on both backends):

```text
-- cargo_items.cargo_document_id
1  1  SENTINEL-BL-KEEP-001  sentinel cargo
2  2  SENTINEL-BL-KEEP-002  second cargo

-- equipment nfc_tag_uid / is_active
1  AST-SENTINEL-ACTIVE    SENTINELNFCUID000000000000000001  1/t  점자-활성
2  AST-SENTINEL-INACTIVE  SENTINELNFCUID000000000000000002  0/f  점자-비가동

-- work_assignments.assignment_id
1  2  2026-01-15  1  1  ELIGIBLE
2  3  2026-01-15  1  1  EXCLUDED
3  2  2026-01-16  2  1  ELIGIBLE

-- gate_verify_logs
1  1     2  1  t/1  SENTINEL_GATE_VERIFY_REASON_KEEP  2026-01-15  t/1
2  NULL  3  1  f/0  SENTINEL_BLOCK_KEEP               2026-01-16  f/0
```

21 sidecar tables present after the populated upgrade on both backends.

SQLite 0005 with FK ON completed. No `DROP TABLE "equipment"` / `DROP TABLE "cargo_documents"`. `cargo_items.cargo_document_id` was not nulled.

## Manual QA SQL

SQLite populated (`PRAGMA foreign_keys=ON`):

```text
cargo_documents|2
equipment|2
equipment_check_logs|2
work_assignments|3
cargo_items|2
gate_verify_logs|2
```

PostgreSQL populated:

```text
 cargo_documents      |     2
 equipment            |     2
 equipment_check_logs |     2
 work_assignments     |     3
 cargo_items          |     2
 gate_verify_logs     |     2
```

Sidecar table list (21 rows on both backends): `cargo_document_versions`, `cargo_item_documents`, `container_cargo_items`, `containers`, `education_completions`, `education_courses`, `education_target_rules`, `equipment_check_events`, `equipment_profiles`, `equipment_tag_tokens`, `gate_events`, `ppe_requirements`, `shared_equipment_claims`, `v2_work_assignments`, `work_assignment_equipment`, `work_ppe_requirement_snapshots`, `work_preparations`, `work_stops`, `work_targets`, `work_types`, `works`.

## Verification

```sh
jq empty apis/api/models/*.json
jq empty apis/api/migrations/0005_sidecar_workflow_v2.vespertide.json
git diff --check
```

`jq empty` exited 0 for every source model and for 0005. `git diff --check` passed. `git diff --name-only -- apis/api/migrations/0001* 0002* 0003* 0004*` empty. `git diff --name-only -- apis/api/models apis/api/src/routes apps .github` empty.

## Adversarial receipts

- **Data loss:** populated SQLite and PostgreSQL kept every required count and sentinel, including `cargo_items.cargo_document_id` and `equipment.nfc_tag_uid`/`is_active`.
- **FK rewrite:** 0005 JSON and v5 SQL contain no parent `CREATE`/`ALTER`/`DROP`/`TEMP`/`DELETE`. The 3E SQLite `DROP TABLE "equipment"` failure path is not present.
- **Malformed:** `jq empty` passed on models and 0005.
- **Stale baseline:** starting HEAD was exactly `4aa758e`.
- **Dirty worktree:** untracked `.omo/evidence/task-2d-stale-evidence-cleanup.md` and `.omo/evidence/task-3e-data-preserving-migration.md` were preserved and excluded from staging.
- **Misleading generator success:** revision JSON was inspected (21 `create_table` only) before export and before applying DBs. Pre-gate SQL already showed the same 21-table additive plan.
- **Interruption / leftovers:** disposable Postgres DBs `task_3f_fresh`/`task_3f_legacy` dropped; `/tmp/task-3f*` harness DBs, SQL captures, export preview, and apply crates removed. No background process left running.
- **Hung:** both runtimes returned `applied` and exited 0.

## Cleanup

- Dropped disposable Postgres DBs `task_3f_fresh`, `task_3f_legacy`
- Removed `/tmp/task-3f`, `/tmp/task-3f-apply-v4`, `/tmp/task-3f-apply-v5`, `/tmp/task-3f-export-preview`, and pre-gate/log SQL captures
- Untracked task-2d / task-3e evidence left untouched
- Source worktree `/Users/wingwogus/Projects/PortDocAI` not edited
