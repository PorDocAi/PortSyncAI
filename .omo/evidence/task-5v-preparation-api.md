# Todo 5V preparation sidecar API evidence

## Scope

Added authenticated sidecar APIs for:

- education courses, target rules, completion records, completion listing, and readiness evaluation;
- PPE requirements, review confirmation, immutable work PPE snapshots, and snapshot queries;
- assignment-scoped work preparation recalculation/query;
- assignment/work-scoped open and closed work stops.

The routes are additive modules registered in `apis/api/src/routes/mod.rs`. They use the generated v2 sidecar entities for work, assignments, education, preparation, and stops. PPE uses a narrow raw-SQL compatibility adapter for the existing generated/migration column spelling (`source_document_version_number` vs the physical `source_document_version`) and maps the result back into a typed sidecar record.

No migration, OpenAPI, UI, CI, legacy route, or attendance model/boolean was changed. The preparation API never writes `attendances.equipment_check_completed`.

## TDD coverage

`apis/api/tests/preparation.rs` was added and runs in-process against a fresh SQLite database with the real handlers and migration stack.

- `education_readiness_distinguishes_missing_expired_and_current`
  - creates a required rule;
  - verifies `MISSING` and `fulfilled=false` before completion;
  - creates an expired completion and verifies `EXPIRED` and `fulfilled=false`;
  - creates a valid completion and verifies `CURRENT` and `fulfilled=true`;
  - verifies course/rule/completion list endpoints.
- `confirmed_ppe_requirement_can_be_snapshotted_and_queried`
  - creates a pending PPE requirement;
  - confirms it through the review endpoint;
  - creates and queries the immutable work snapshot;
  - verifies the legacy attendance equipment boolean is unchanged.
- `work_stop_round_trips_open_and_closed_state`
  - creates an assignment-scoped `OPEN` stop (`fulfilled=false`);
  - queries it;
  - closes it and verifies `CLOSED` (`fulfilled=true`) through query.
- `preparation_is_blocked_by_open_stop_and_ready_after_close`
  - establishes current education and PPE snapshot state;
  - verifies an open stop produces `BLOCKED` preparation;
  - closes the stop and verifies `READY`/`fulfilled=true`;
  - queries the saved preparation.

## Verification

- `cargo test -p api --test preparation --no-fail-fast`
  - PASS: 4 passed, 0 failed.
- `cargo build -p api`
  - PASS.
- Real Vespera route collection was built and generated paths were inspected:
  - `/education/courses`
  - `/education/rules`
  - `/education/completions`
  - `/education/readiness`
  - `/ppe/requirements`
  - `/ppe/requirements/{id}/confirm`
  - `/ppe/snapshots`
  - `/work-preparations`
  - `/work-preparations/{id}`
  - `/work-stops`
  - `/work-stops/{id}/close`
- `rustfmt --edition 2024 --check` on every touched route/test file
  - PASS.
- Repository-wide `cargo fmt --all -- --check`
  - reports pre-existing generated-model formatting drift in `apis/api/src/models/*.rs`; no generated model was touched.
- Rust LSP diagnostics were unavailable because `rust-analyzer` is not installed.

## Manual curl QA

A temporary SQLite-backed API process was started with `JWT_SECRET=task5v-secret` and a temporary port/database. Minimal department, role, employee, work type, work, and v2 assignment bridge rows were seeded with `sqlite3`.

Observed responses:

```text
health={"status":"ok"}
valid_completion={"fulfilled":true,"education_course_id":1,"employee_id":1}
readiness={"fulfilled":false,"statuses":["CURRENT","EXPIRED"]}
open_stop={"status":"OPEN","fulfilled":false}
closed_stop={"status":"CLOSED","fulfilled":true}
queried_stop={"status":"CLOSED","fulfilled":true}
```

The temporary process, database, and logs were removed after QA.

## OpenAPI dirt restoration

`cargo test`/`cargo build` generated changes in `apps/front/openapi.json` and `apps/admin/openapi.json`; both were restored immediately after verification and remain unchanged in the task diff.

Pre-existing untracked evidence files were preserved:

- `.omo/evidence/task-2d-stale-evidence-cleanup.md`
- `.omo/evidence/task-3e-data-preserving-migration.md`
- `.omo/evidence/task-3f1-independent-runtime-replay.md`
