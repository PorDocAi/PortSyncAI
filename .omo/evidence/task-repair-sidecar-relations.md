# Sidecar relation collision repair

- Baseline command: `cd apis/api && cargo test -p api --test work_assignment`
- Baseline result: failed before the source rename with `error[E0124]: field source_document_version is already declared` in both generated sidecar models (`ppe_requirements.rs` and `work_ppe_requirement_snapshots.rs`). The duplicate was between the scalar field and the SeaORM relation field.
- Source repair: renamed only the scalar schema columns in `apis/api/models/ppe_requirements.json` and `apis/api/models/work_ppe_requirement_snapshots.json` from `source_document_version` to `source_document_version_number`. The FK column `source_document_version_id` was preserved unchanged.
- Regeneration: `cd apis/api && vespertide export --orm seaorm`
- Formatting: `cargo fmt --all`
- Verification: `cargo test -p api --test work_assignment`
- Verification result: passed, 6 tests, 0 failed. Generated entities now contain `source_document_version_number` for the scalar and retain `source_document_version` only for the relation.

Pre-existing untracked evidence files were preserved and are not included in this repair commit.
