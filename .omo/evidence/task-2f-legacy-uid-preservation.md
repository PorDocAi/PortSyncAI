# Todo 2F - legacy equipment UID preservation

## Baseline / failing-first evidence

At revision `19b9e9a`, abortable generator revision inspection reported the legacy UID as a destructive change:

```text
Resolve drop: column `equipment.nfc_tag_uid` (char(32) UNIQUE NOT NULL)
```

The generator wanted to drop/rename `equipment.nfc_tag_uid`; that prompt is a data-loss and compatibility hazard. Historical migration `0001_init_schema.vespertide.json` is the baseline contract: `char(32)`, `nullable: false`, `unique: true`, comment `NFC 태그 UID (FR-D2)`.

## Contract restored

`apis/api/models/equipment.json` retains `equipment.nfc_tag_uid` exactly beside additive v2 ownership/lifecycle fields and `equipment_tag_tokens`. The UID is historical audit/compatibility data only. It is never a v2 request API field and is never copied, derived, or hashed into `equipment_tag_tokens.tag_token_hash`; v2 tokens are independently random opaque values.

The conversion map records the retained legacy field and explicitly has no v2 token target. No migration, generated entity, route, or OpenAPI output was created or changed.

## Checker / literal JSON QA

```sh
jq '.columns[] | select(.name == "nfc_tag_uid" or .name == "tag_token_hash") | {name,type,nullable,unique}' apis/api/models/equipment.json apis/api/models/equipment_tag_tokens.json
python3 .omo/evidence/task-2-validate-schema-v2.py
```

Expected checker output:

```text
schema-v2 contract: PASS
```

The literal model must show `nfc_tag_uid` as `char(32)`, NOT NULL, unique, while `tag_token_hash` remains a separate unique `char(64)` field. The checker rejects UID deletion, malformed type/nullability, or loss of uniqueness.

## Revision inspection and adversarial QA

After restoration, abortable revision inspection no longer prompts to drop or rename `equipment.nfc_tag_uid`. Manual QA checks the literal field and prompt text. The contract rejects UID data loss, malformed/stale token substitutions, dirty or misleading conversion claims, and interruption paths that would accept generated migration/entity/route/OpenAPI artifacts. Existing untracked `.omo/evidence/task-2d-stale-evidence-cleanup.md` is intentionally not staged.
