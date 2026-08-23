# Gate v2 readiness evidence

Implemented v2 assignment readiness evaluation in the gate verify path.

## Behavior

- Accepts optional `v2_work_assignment_id` in gate verification requests.
- Evaluates PPE snapshot allocation, required education completion, and open work stops.
- Records one `gate_events` row for every v2 verification call, with PASS/BLOCK decision and stable reason codes.
- Test request helpers now preserve v2 JSON request bodies.
- PPE snapshot fixture/query paths use the columns available in the migrated test schema.

## Verification

Command:

```text
cargo test -p api --test gate_v2 --test gate_flow --test auth_terminal
```

Result: 19 passed, 0 failed.

- gate_v2: 3 passed
- gate_flow: 9 passed
- auth_terminal: 7 passed
