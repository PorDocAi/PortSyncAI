# Task 2A evidence: additive legacy preservation

## Scope and baseline

- Worktree: `/Users/wingwogus/Projects/PortDocAI-schema-v2-nfc-contract`
- Branch: `refactor/#29-schema-v2-nfc-contract`
- Starting HEAD: `bb491cf682a241213aa1fe64669afd6541936bbb`
- PR #48 ancestry: `git merge-base --is-ancestor 320624462e9416648b056820f8ea4ef77f7d6ecc HEAD` remains valid because this commit is a child of `bb491cf`.
- Source worktree `/Users/wingwogus/Projects/PortDocAI` was not edited. Its dirty CRLF/untracked paths (`apis/api/Cargo.toml`, `apis/api/models/cargo_documents.json`, `.gjc/`, `.omo/`, `.omx/`) remain outside this task worktree.
- Historical migrations `0001`-`0004` were not modified.
- No SeaORM entity, route, OpenAPI, or `0005+` migration was generated.

## Previous destructive generator record (preserved)

Todo 3 previously attempted `vespertide revision` against the Todo 2 source and was aborted after destructive confirmation. That record remains in `/Users/wingwogus/Projects/PortDocAI/.omo/start-work/ledger.jsonl`:

```json
{"event":"schema-correction-discovered","plan":".omo/plans/schema-v2-nfc-contract.md","task":"2A","session_id":"senpi:01a02a10-623c-70b8-8356-82d355f5c82e","commands":["vespertide migration generator attempted in Todo 3"],"artifact":".omo/evidence/task-2-pr48-conversion-map.md","adversarial_classes":{"data_loss":"blocked generator prompts would recreate work_assignments and drop gate_verify_logs/cargo_items.cargo_document_id; destructive choices rejected"},"cleanup":"no generated file or temp repo artifact remained","decision":"additive legacy preservation must precede migration generation"}
```

The live pre-2A plan, captured with `vespertide diff` and `vespertide sql --backend sqlite` from `bb491cf` before model edits, still contained those three surfaces:

```text
19. Delete table: gate_verify_logs
32. Delete column: cargo_items.cargo_document_id
63. Delete column: work_assignments.assignment_id
64. Delete column: work_assignments.cargo_item_id
65. Delete column: work_assignments.eligibility_status
66. Delete column: work_assignments.work_date
72. Add column: work_assignments.work_assignment_id
73. Add column: work_assignments.work_id
78. Add constraint: FK (work_id) -> works on work_assignments
```

SQLite SQL for the required `work_id` replacement recreated the table and filled the new PK/FK with `NULL`:

```text
Action: AddColumn: work_assignments.work_id
73-2. INSERT INTO "work_assignments_temp" (..., "work_assignment_id", "work_id") SELECT ..., NULL AS "work_id" FROM "work_assignments"
73-3. DROP TABLE "work_assignments"
Action: DeleteTable: gate_verify_logs
19. DROP TABLE "gate_verify_logs"
Action: DeleteColumn: cargo_items.cargo_document_id
```

VesperTide 0.2.1 `revision` treats a required FK add as table recreation (`adding required FK column` / `ALL DATA in these tables will be DELETED`) and omitted tables/columns as drop resolutions. Those prompts were never confirmed.

## Failing-first contract check

Checker assertions were updated first so omitted legacy surfaces and a non-null `v2_work_id` fail. Against the pre-restoration `bb491cf` models the updated checker exited `1`:

```text
schema-v2 contract: FAIL (18 errors)
- missing model: gate_verify_logs
- work_assignments: missing column assignment_id
- work_assignments: missing column cargo_item_id
- work_assignments: missing column eligibility_status
- work_assignments: missing column v2_work_id
- work_assignments: missing column work_date
- cargo_items: missing column cargo_document_id
- work_assignments: unique uq_work_assignment_work_employee must cover ['employee_id', 'v2_work_id']
- work_assignments: legacy assignment_id primary key must be retained
- work_assignments: work_assignment_id is a required replacement PK and must not replace assignment_id
- work_assignments: required work_id replacement must not exist; use nullable v2_work_id
- work_assignments: missing retained legacy column work_date
- work_assignments: missing retained legacy column cargo_item_id
- work_assignments: missing retained legacy column eligibility_status
- cargo_items: legacy cargo_document_id direct FK must be retained
- conversion map missing anchor: retained legacy
- conversion map missing anchor: v2 counterpart
- conversion map missing anchor: v2_work_id
```

## Additive restorations

1. `work_assignments.json` keeps `assignment_id`, `work_date`, `cargo_item_id`, and `eligibility_status`. New v2 work is a separate nullable `v2_work_id` FK to `works.work_id`. `UNIQUE(v2_work_id, employee_id)` is additive. Additive v2 tables now reference `work_assignments.assignment_id` instead of a renamed PK.
2. `gate_verify_logs.json` was added with the exact 0003 column set (`verify_log_id`, `attendance_id`, `employee_id`, `terminal_id`, `allowed`, `reason`, `work_date`, `is_pass_event`, `created_at`) including `uq_attendance_workdate_passed`. `gate_events` remains additive with nullable `legacy_verify_log_id`.
3. `cargo_items.cargo_document_id` was restored as the nullable direct FK. `cargo_item_documents` remains the additive v2 counterpart.
4. `.omo/evidence/task-2-pr48-conversion-map.md` now classifies those three as retained legacy plus v2 counterparts, not dropped replacements.

## Automated validation

Commands:

```sh
jq empty apis/api/models/*.json
python3 .omo/evidence/task-2-validate-schema-v2.py
```

Results:

- `jq empty`: PASS for all `45` source JSON files.
- Contract checker:

```text
schema-v2 contract: PASS
validated 23 required #29/#34 source models
validated retained legacy assignment/gate/cargo surfaces plus additive v2 counterparts
validated work assignment, token hash, idempotency, claim, and gate-event identities
validated PR #48 conversion map anchors
```

`gate_verify_logs` source columns compared to `0003_add_gate_terminals_and_verify_logs.vespertide.json` matched name, type, nullability, PK, FK, index, unique, default, and comment. `git diff --check` was clean. `git diff --name-only -- apis/api/migrations` was empty.

## Literal jq inspection (manual QA)

```sh
jq -r '
  def cols: [.columns[].name];
  select(.name == "work_assignments" or .name == "cargo_items" or .name == "gate_verify_logs" or .name == "gate_events" or .name == "cargo_item_documents") |
  {table: .name, columns: cols}
' apis/api/models/*.json
```

Observed counterparts:

- `work_assignments.columns` includes retained `assignment_id`, `work_date`, `cargo_item_id`, `eligibility_status` and additive nullable `v2_work_id`.
- `cargo_items.columns` includes retained `cargo_document_id` and additive `cargo_item_documents` still exists.
- `gate_verify_logs.columns` equals the 0003 list; `gate_events` still has nullable unique `legacy_verify_log_id`.

## Abortable VesperTide planned-change capture

Commands (no migration written):

```sh
vespertide diff
vespertide sql --backend sqlite
vespertide sql --backend postgres
# disposable PTY: vespertide revision -m "abortable 2A planned-change capture"
# SIGINT/SIGTERM after first prompt; no confirmation accepted
```

Post-restoration `vespertide diff` no longer lists the three 2A destructive surfaces:

```text
ABSENT Delete table: gate_verify_logs
ABSENT Delete column: cargo_items.cargo_document_id
ABSENT Delete column: work_assignments.assignment_id
ABSENT Add column: work_assignments.work_id
ABSENT Add column: work_assignments.work_assignment_id
65. Add column: work_assignments.v2_work_id
69. Add constraint: FK (v2_work_id) -> works on work_assignments
```

Postgres SQL is additive on those surfaces (`ALTER TABLE "work_assignments" ADD COLUMN "v2_work_id" bigint` plus a nullable FK). It does not `DROP TABLE "gate_verify_logs"`, drop `cargo_items.cargo_document_id`, or recreate `work_assignments` for a required `work_id`.

The abortable PTY `revision` first prompt was the expected remaining equipment drop (`equipment.is_active`), not the three 2A surfaces:

```text
⚠ Resolve drop: column `equipment.is_active` (boolean NOT NULL)
  Drop permanently (data lost, irreversible)
  Cancel migration
ABSENT The following tables need to be RECREATED
ABSENT adding required FK column
ABSENT Resolve drop: table `gate_verify_logs`
ABSENT Resolve drop: column `cargo_items.cargo_document_id`
ABSENT Resolve drop: column `work_assignments.assignment_id`
```

The process was killed with SIGINT/SIGTERM. No confirmation was sent. `apis/api/migrations` still contains only `0001`-`0004`.

## Remaining limitation (out of 2A scope)

SQLite SQL still rewrites `work_assignments` when adding the non-null `status` enum with a default (`CREATE TABLE work_assignments_temp` then `DROP TABLE "work_assignments"`). That rewrite copies `assignment_id`, `work_date`, `cargo_item_id`, and `eligibility_status`; it is not the required-FK recreation that previously filled `work_id` with `NULL`. Postgres uses `ADD COLUMN "status" ... NOT NULL DEFAULT ('ASSIGNED')` without dropping the table. Unique-addition SQL still emits a `DELETE FROM "work_assignments"` dedupe for `UNIQUE(employee_id, v2_work_id)` because all current `v2_work_id` values are null. Todo 3B must choose a unique strategy that does not delete legacy rows.

## Adversarial and cleanup

- Malformed JSON: `jq empty apis/api/models/*.json` parsed every model.
- Data-loss/destructive prompt: previous recreation/drop plan is preserved above; the abortable retry did not propose those three drops and was not confirmed.
- Dirty worktree: task worktree started clean at `bb491cf`; source worktree dirty paths were left untouched.
- Stale base: started from `bb491cf`, the Todo 2 commit.
- Misleading generator success: `revision` was aborted before write; no `0005` file exists.
- Interruption: PTY was signalled after the first prompt; leftover `/tmp/vespertide-*.txt` captures were discarded after evidence transcription.
- Cleanup receipt: generator aborted, no migration/entity/temp repo artifact remained in the worktree.
