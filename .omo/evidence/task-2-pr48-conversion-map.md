# Todo 2G: PR #48 to additive workflow v2 conversion map

## Physical transition boundary

The **immutable legacy physical tables** are `cargo_documents`, `cargo_items`, `equipment`, `equipment_check_logs`, `work_assignments`, `attendances`, `gate_terminals`, and `gate_verify_logs`. Their model columns and constraints match migrations `0001`-`0004` exactly. V2 does not add, remove, rename, retype, re-comment, or re-index their columns; this prevents VesperTide's SQLite table-rebuild path from dropping FK parents.

The legacy rows remain queryable for compatibility and audit, but new workflow behavior is **v2 authoritative** in additive tables. Nullable unique `legacy_*` foreign keys identify imported rows without requiring a v2 row for every legacy row. V2-native rows may leave those bridges null.

## Source-to-target conversion

| Source | Target | Conversion | Status |
| --- | --- | --- | --- |
| `cargo_documents.cargo_document_id` | `cargo_document_versions.legacy_cargo_document_id` | Create at most one initial v2 version for each converted legacy document; preserve the source ID through the nullable unique FK. | Retained legacy parent plus v2 authoritative version. |
| `cargo_documents.document_type` | `cargo_document_versions.document_type` | Copy `BL`/`DGD`; v2-native versions may also use `CI`/`MSDS`. | Retained legacy field; expanded type exists only in the sidecar. |
| `cargo_documents.review_status` | `cargo_document_versions.review_status` | Copy `CONFIRMED -> CONFIRMED`, otherwise `PENDING`; set `document_version=1`; do not fabricate reviewer provenance. | Retained legacy review plus v2 authoritative review/version. |
| absent in PR #48 documents | `cargo_document_versions.processing_status`, `reviewed_by_id`, `reviewed_at` | Use an explicit migration value for processing status and null reviewer fields. | V2-only state. |
| `cargo_items.cargo_document_id` | `cargo_item_documents(cargo_item_id,cargo_document_version_id,document_role)` | Preserve the direct legacy FK unchanged. When its document was imported, add one role relation to that v2 version. | Retained legacy relation plus additive roles. |
| `work_assignments.assignment_id` | `v2_work_assignments.legacy_assignment_id` | Preserve legacy identity through a nullable unique FK. A converted v2 assignment receives its own `work_assignment_id`. | Retained legacy assignment plus v2 authoritative assignment. |
| `work_assignments.employee_id`, `assigned_by_id` | `v2_work_assignments.employee_id`, `assigned_by_id` | Copy only when an explicit v2 work is created; do not alter the source row. | Retained legacy values plus v2 linkage. |
| `work_assignments.work_date`, `cargo_item_id` | `works` and `work_targets` | Preserve source fields. A later conversion may create a work/target when mapping is provable; no parent column is added. | Retained legacy fields; no `v2_work_id` on the parent. |
| absent in PR #48 assignments | `v2_work_assignments.status`, `selected_at`, `started_at`, `completed_at` | Initialize converted rows fail-closed as `ASSIGNED`; leave lifecycle timestamps null. | V2-only assignment state. |
| `equipment.equipment_id` | `equipment_profiles.legacy_equipment_id` | Create at most one profile for each imported asset and preserve the source ID through the nullable unique FK. | Retained legacy asset plus v2 authoritative profile. |
| `equipment.equipment_type_id`, `asset_number`, `braille_label` | corresponding `equipment_profiles` fields | Copy when a legacy asset is imported; do not alter the physical `equipment` row. | Retained legacy values plus v2 profile values. |
| `equipment.is_active` | retained `equipment.is_active` only | Keep exact historical state; never translate it into v2 availability. | Retained legacy field. |
| `equipment.nfc_tag_uid` | retained `equipment.nfc_tag_uid` only | Preserve exactly for audit/compatibility. Never hash, derive, or copy it into a v2 token. | Retained legacy identifier, not a v2 credential. |
| absent in PR #48 equipment | `equipment_profiles.ownership_type`, `owner_employee_id`, `status` | Imported assets become `SHARED`, ownerless, and `BLOCKED` until administrative review. | V2 authoritative ownership/lifecycle. |
| absent in PR #48 equipment | `equipment_tag_tokens.equipment_profile_id`, `tag_token_hash` | Create no token during conversion. Independently random opaque tokens are issued later and only their hash is stored. | V2-only token lifecycle. |
| `equipment_check_logs.*` | retained `equipment_check_logs.*` | Preserve every row and the historical `uq_equipment_workdate` exactly. Do not manufacture an assignment-scoped event. | Retained legacy audit table. |
| absent in PR #48 checks | `equipment_check_events`, `work_assignment_equipment`, `shared_equipment_claims` | New requests use v2 assignment/profile IDs for idempotent events, accepted allocations, and portable active claims. | V2 authoritative check/allocation/claim state. |
| `attendances.*` | retained `attendances.*` | Preserve all 0001 and 0004 fields, including readiness summaries and work status, without changing the table. | Retained legacy attendance parent. |
| `attendances.work_status=STOPPED` | `work_stops.legacy_attendance_id` | Add an `OPEN` stop with nullable FK to the source attendance; leave `work_id`, `work_assignment_id`, and actor null when not provable. | Retained legacy summary plus v2 authoritative stop. |
| `attendances.approval_status`, `instruction_ack_completed`, `equipment_check_completed` | legacy attendance/approval/history only | Do not grant v2 readiness. | Retained legacy audit, removed from v2 authority. |
| absent in PR #48 readiness | `work_preparations.work_assignment_id` | New preparation state belongs only to `v2_work_assignments`. | V2 authoritative preparation. |
| `gate_terminals.*` | retained `gate_terminals.*` and `gate_events.terminal_id` | Preserve credential rows exactly; new events may reference the legacy terminal ID. | Retained legacy physical parent. |
| `gate_verify_logs.verify_log_id` | `gate_events.legacy_verify_log_id` | Preserve the source log and optionally add one v2 event with a nullable unique FK to it. | Retained legacy log plus v2 authoritative event. |
| `gate_verify_logs.attendance_id` | `gate_events.legacy_attendance_id` | Preserve the source FK and copy it only as nullable event provenance; do not infer a v2 assignment. | Retained legacy link plus audit bridge. |
| `gate_verify_logs.employee_id`, `terminal_id`, `work_date`, `created_at` | corresponding `gate_events` values | Copy exact identities/time when converting; `work_date` remains legacy audit metadata. | Retained legacy values plus event provenance. |
| `gate_verify_logs.allowed` | `gate_events.decision` | Map `true -> PASS`, `false -> BLOCK`. | V2 authoritative decision. |
| `gate_verify_logs.reason` | `gate_events.reason_code`, `reason_codes`, `response_json` | Preserve exact text in response JSON; map known text to stable codes and unknown text to `LEGACY_GATE_REASON`. | V2 authoritative structured reason. |
| absent in PR #48 gate logs | `gate_events.work_assignment_id`, `idempotency_key`, `request_fingerprint` | Leave null for imported rows. New v2 requests populate v2 assignment and idempotency fields. | V2-only request identity. |

## Additive v2 identity and constraint contract

- `cargo_document_versions` owns expanded document types, processing, review, and version state. `legacy_cargo_document_id` is nullable, unique, and references the immutable parent.
- `v2_work_assignments` owns work linkage and lifecycle. `UNIQUE(work_id, employee_id)` applies only here; `legacy_assignment_id` is nullable, unique, and references the immutable parent.
- `equipment_profiles` owns type/profile/ownership/status state. `legacy_equipment_id` is nullable, unique, and references the immutable parent.
- `equipment_tag_tokens`, `work_assignment_equipment`, `shared_equipment_claims`, and `equipment_check_events` reference v2 profile/assignment IDs, not legacy parents as active workflow identities.
- `work_preparations`, assignment-scoped `work_stops`, and `gate_events` reference `v2_work_assignments.work_assignment_id`.
- Legacy provenance in `work_stops`/`gate_events` uses nullable FKs to `attendances.attendance_id` and `gate_verify_logs.verify_log_id`.
- `equipment_check_events` remains append-only with `UNIQUE(employee_id,idempotency_key)` and exact response replay state.
- `shared_equipment_claims` remains a portable `UNIQUE(equipment_profile_id)` active-claim lock.
- `gate_events` remains append-only with immutable `gate_event_id` and `UNIQUE(terminal_id,idempotency_key)` for new requests.
