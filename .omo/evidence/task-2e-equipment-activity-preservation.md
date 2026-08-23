# Todo 2E - equipment activity preservation

## Baseline / failing-first evidence

At revision `dcc9c02`, the abortable generator revision inspection prompted for the remaining legacy equipment activity change:

```text
Resolve drop: column `equipment.is_active` (boolean NOT NULL)
```

This was the generator's proposed drop/rename hazard. No generated migration, entity, route, or OpenAPI output was accepted. Historical migration `0001_init_schema.vespertide.json` confirms the exact source contract: `type: boolean`, `nullable: false`, `default: true`, comment `사용 가능 여부`.

## Contract restored

`apis/api/models/equipment.json` now retains `equipment.is_active` exactly and keeps additive `ownership_type`, `owner_employee_id`, `status`, and status metadata. The conversion map explicitly maps `equipment.is_active` to retained `equipment.is_active` plus `equipment.status`; it is not renamed or dropped. Legacy activity is preserved for compatibility/audit, while v2 migration initializes lifecycle `status` to `BLOCKED` pending review.

## Checker / literal JSON QA

```sh
python3 .omo/evidence/task-2-validate-schema-v2.py
jq '.columns[] | select(.name == "is_active" or .name == "status") | {name,type,nullable,default}' apis/api/models/equipment.json
```

Expected checker output:

```text
schema v2 validation: PASS
```

The literal fields show `is_active` as boolean, non-null, default `true`, and `status` as the v2 lifecycle enum; both are present in the same equipment model. The checker rejects missing coexistence or any change to the legacy type/nullability/default.

## Revision inspection / cleanup

Abortable generator inspection completed without a prompt to drop or rename `equipment.is_active` after the contract fix. No temporary captures or generated artifacts were retained. Existing untracked `.omo/evidence/task-2d-stale-evidence-cleanup.md` was intentionally not staged.
