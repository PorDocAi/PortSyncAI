# Todo 2B: defer nullable v2 assignment uniqueness

## Scope and safety

This change removes only the schema-level `UNIQUE(employee_id, v2_work_id)` from `apis/api/models/work_assignments.json`. The retained `assignment_id` primary key, legacy columns, nullable `v2_work_id` foreign key, and ordinary `v2_work_id` index are unchanged. Migrations `0001`-`0004`, generated migrations, entities, routes, and OpenAPI were not modified.

## Failing-first baseline

Before the model edit, `cd apis/api && vespertide sql --backend sqlite` produced the destructive dedupe:

```text
66-1. DELETE FROM "work_assignments" WHERE "assignment_id" NOT IN (SELECT MIN("assignment_id") FROM "work_assignments" GROUP BY "employee_id", "v2_work_id")
66-2. CREATE UNIQUE INDEX "uq_work_assignments__uq_work_assignment_work_employee" ON "work_assignments" ("employee_id", "v2_work_id")
```

The captured baseline is `/tmp/task-2b-baseline-sqlite.txt`; existing task-3B raw SQL evidence remains preserved in `.omo/evidence/task-3b-vespertide-sql-sqlite.txt` and `.omo/evidence/task-3b-vespertide-sql-postgres.txt`.

## Contract decision

Nullable v2 uniqueness is intentionally deferred to future application-level transactional validation after assignment activation establishes a non-null `v2_work_id`. This avoids schema-generation deduplication and data loss while legacy rows are still nullable. The Todo 2 checker rejects reintroduction of `uq_work_assignment_work_employee`, and the conversion map records the same deferred-validation contract.

## Verification

- `jq empty apis/api/models/work_assignments.json`: pass.
- `python3 .omo/evidence/task-2-validate-schema-v2.py`: pass.
- SQLite and PostgreSQL `vespertide diff`/`sql` output contains no `DELETE FROM "work_assignments"`.
- Manual captured-SQL grep for `DELETE FROM "work_assignments"`: 0.
- Staged diff is limited to this evidence file, the work_assignments model, the Todo 2 checker, and conversion map.
