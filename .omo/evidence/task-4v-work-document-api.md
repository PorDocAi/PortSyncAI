# Task 4V work/document API evidence

- Extended existing POST /work-assignments validation.
- When a cargo item has an MSDS cargo_item_documents bridge, cargo_document_versions.review_status must be CONFIRMED.
- Legacy cargo_documents review validation remains in place when no v2 bridge exists.
- Tests added for mapped unconfirmed (422, MSDS_REVIEW_NOT_CONFIRMED) and confirmed (201).
- Verification: cargo test -p api --test work_assignment attempted; blocked by pre-existing duplicate source_document_version model errors in ppe_requirements.rs and work_ppe_requirement_snapshots.rs.
- cargo fmt --all -- --check reports pre-existing formatting drift across generated/model files.
