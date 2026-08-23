# Todo 2C: guard nullable v2 uniqueness by columns

## Failing-first negative evidence

A temporary copy of the source models was changed to assign the arbitrary name `arbitrary_renamed_composite_unique` to the composite unique group containing `employee_id` and `v2_work_id`. The checker rejected the renamed constraint by its columns, not by its name:

```text
schema-v2 contract: FAIL (1 errors)
- work_assignments: composite unique arbitrary_renamed_composite_unique must not contain employee_id and v2_work_id
exit:1
```

The temporary copy was removed after the check.

## Verification

The unchanged HEAD contract remains valid:

```text
schema-v2 contract: PASS
validated 23 required #29/#34 source models
validated retained legacy assignment/gate/cargo surfaces plus additive v2 counterparts
validated work assignment, token hash, idempotency, claim, and gate-event identities
validated PR #48 conversion map anchors
```

`jq empty apis/api/models/work_assignments.json` passed. The checker now rejects any unique group represented by column-level groups or model constraints when its columns include both `employee_id` and `v2_work_id`, regardless of constraint name. No model semantics, migrations, entities, routes, OpenAPI, or CI were changed.
