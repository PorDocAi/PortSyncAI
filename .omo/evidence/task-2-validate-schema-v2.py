#!/usr/bin/env python3
"""Validate the source-model inventory required by issues #29 and #34."""

from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
MODELS_DIR = ROOT / "apis/api/models"
MAPPING_PATH = ROOT / ".omo/evidence/task-2-pr48-conversion-map.md"

REQUIRED_TABLES = {
    "containers",
    "container_cargo_items",
    "cargo_item_documents",
    "cargo_documents",
    "work_types",
    "works",
    "work_targets",
    "work_assignments",
    "education_courses",
    "education_target_rules",
    "education_completions",
    "ppe_requirements",
    "work_ppe_requirement_snapshots",
    "equipment",
    "equipment_tag_tokens",
    "work_assignment_equipment",
    "shared_equipment_claims",
    "equipment_check_events",
    "work_preparations",
    "work_stops",
    "gate_terminals",
    "gate_events",
}

REQUIRED_COLUMNS = {
    "container_cargo_items": {"container_id", "cargo_item_id"},
    "cargo_item_documents": {"cargo_item_id", "cargo_document_id", "document_role"},
    "cargo_documents": {
        "document_type",
        "processing_status",
        "review_status",
        "document_version",
        "reviewed_by_id",
        "reviewed_at",
    },
    "works": {"work_type_id", "status", "scheduled_start_at", "scheduled_end_at"},
    "work_targets": {"work_id", "target_type", "container_id", "cargo_item_id"},
    "work_assignments": {
        "work_assignment_id",
        "work_id",
        "employee_id",
        "status",
        "selected_at",
    },
    "education_target_rules": {"education_course_id", "work_type_id", "dg_class_id"},
    "education_completions": {
        "education_course_id",
        "employee_id",
        "completed_at",
        "expires_at",
    },
    "ppe_requirements": {
        "equipment_type_id",
        "category",
        "performance_criteria",
        "source_text",
        "source_document_id",
        "source_document_version",
        "reviewed_by_id",
        "reviewed_at",
        "review_status",
    },
    "work_ppe_requirement_snapshots": {
        "work_id",
        "ppe_requirement_id",
        "equipment_type_id",
        "category",
        "performance_criteria",
        "source_document_id",
        "source_document_version",
        "reviewed_by_id",
        "snapshotted_at",
    },
    "equipment": {
        "ownership_type",
        "owner_employee_id",
        "status",
        "manufacturer_replacement_due_at",
    },
    "equipment_tag_tokens": {
        "equipment_id",
        "tag_token_hash",
        "is_active",
        "issued_by_id",
        "issued_at",
        "deactivated_by_id",
        "deactivated_at",
    },
    "work_assignment_equipment": {
        "work_assignment_id",
        "equipment_id",
        "requirement_snapshot_id",
        "accepted_at",
    },
    "shared_equipment_claims": {"work_assignment_id", "equipment_id", "claimed_at"},
    "equipment_check_events": {
        "employee_id",
        "work_assignment_id",
        "equipment_id",
        "idempotency_key",
        "request_fingerprint",
        "http_status",
        "reason_code",
        "response_json",
        "client_scanned_at",
    },
    "work_preparations": {"work_assignment_id", "status", "prepared_at"},
    "work_stops": {
        "work_id",
        "work_assignment_id",
        "legacy_attendance_id",
        "status",
        "stopped_by_id",
        "stopped_at",
        "closed_by_id",
        "closed_at",
    },
    "gate_terminals": {"gate_id", "token_hash", "is_active"},
    "gate_events": {
        "gate_event_id",
        "terminal_id",
        "gate_id",
        "work_assignment_id",
        "employee_id",
        "idempotency_key",
        "request_fingerprint",
        "decision",
        "reason_code",
        "reason_codes",
        "decision_input_versions",
        "occurred_at",
        "legacy_verify_log_id",
        "legacy_attendance_id",
        "legacy_work_date",
    },
}

REQUIRED_ENUM_VALUES = {
    ("cargo_documents", "document_type"): {"BL", "DGD", "CI", "MSDS"},
    ("equipment", "ownership_type"): {"PERSONAL", "SHARED"},
    ("equipment", "status"): {"AVAILABLE", "BLOCKED", "DAMAGED", "LOST", "REPLACED"},
    ("works", "status"): {"PLANNED", "ACTIVE", "STOPPED", "COMPLETED", "CANCELLED"},
    ("gate_events", "decision"): {"PASS", "BLOCK"},
}

REQUIRED_UNIQUES = {
    "work_assignments": [
        ("uq_work_assignment_work_employee", {"work_id", "employee_id"}),
    ],
    "equipment_check_events": [
        ("uq_equipment_check_employee_idempotency", {"employee_id", "idempotency_key"}),
    ],
    "work_assignment_equipment": [
        ("uq_assignment_equipment", {"work_assignment_id", "equipment_id"}),
    ],
    "shared_equipment_claims": [
        ("uq_shared_equipment_active_claim", {"equipment_id"}),
    ],
    "gate_events": [
        ("uq_gate_event_terminal_idempotency", {"terminal_id", "idempotency_key"}),
    ],
}

MAPPING_ANCHORS = {
    "gate_terminals",
    "gate_verify_logs",
    "cargo_documents.review_status",
    "attendances.work_status",
    "attendances.approval_status",
    "attendances.instruction_ack_completed",
    "attendances.equipment_check_completed",
    "equipment_check_logs",
    "equipment.nfc_tag_uid",
    "source",
    "target",
    "conversion",
    "dropped legacy field",
}


def load_models() -> tuple[dict[str, dict], list[str]]:
    models: dict[str, dict] = {}
    errors: list[str] = []
    for path in sorted(MODELS_DIR.glob("*.json")):
        try:
            model = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"{path.name}: invalid JSON: {error}")
            continue
        name = model.get("name")
        if not isinstance(name, str):
            errors.append(f"{path.name}: missing string model name")
            continue
        if name in models:
            errors.append(f"duplicate model name: {name}")
        models[name] = model
    return models, errors


def columns(model: dict) -> dict[str, dict]:
    return {column["name"]: column for column in model.get("columns", [])}


def unique_groups(model: dict) -> dict[str, set[str]]:
    groups: dict[str, set[str]] = {}
    for column in model.get("columns", []):
        unique = column.get("unique")
        names = unique if isinstance(unique, list) else [unique]
        for name in names:
            if isinstance(name, str):
                groups.setdefault(name, set()).add(column["name"])
    for constraint in model.get("constraints", []):
        if constraint.get("type") == "unique" and constraint.get("name"):
            groups[constraint["name"]] = set(constraint.get("columns", []))
    return groups


def validate() -> list[str]:
    models, errors = load_models()

    for table in sorted(REQUIRED_TABLES - models.keys()):
        errors.append(f"missing model: {table}")

    for table, required in REQUIRED_COLUMNS.items():
        if table not in models:
            continue
        actual = columns(models[table])
        for column in sorted(required - actual.keys()):
            errors.append(f"{table}: missing column {column}")

    for (table, column_name), required in REQUIRED_ENUM_VALUES.items():
        if table not in models or column_name not in columns(models[table]):
            continue
        column_type = columns(models[table])[column_name].get("type")
        actual = set(column_type.get("values", [])) if isinstance(column_type, dict) else set()
        if not required <= actual:
            errors.append(
                f"{table}.{column_name}: missing enum values {sorted(required - actual)}"
            )

    for table, expected_groups in REQUIRED_UNIQUES.items():
        if table not in models:
            continue
        groups = unique_groups(models[table])
        for name, expected_columns in expected_groups:
            if groups.get(name) != expected_columns:
                errors.append(
                    f"{table}: unique {name} must cover {sorted(expected_columns)}"
                )

    equipment = models.get("equipment")
    if equipment and "nfc_tag_uid" in columns(equipment):
        errors.append("equipment: legacy nfc_tag_uid must not remain in the v2 source")

    tokens = models.get("equipment_tag_tokens")
    if tokens:
        token_hash = columns(tokens).get("tag_token_hash", {})
        if token_hash.get("type") != {"kind": "char", "length": 64}:
            errors.append("equipment_tag_tokens.tag_token_hash must be a 64-character hash")
        if token_hash.get("unique") is not True:
            errors.append("equipment_tag_tokens.tag_token_hash must be unique")

    legacy_logs = models.get("equipment_check_logs")
    if legacy_logs and "uq_equipment_workdate" in unique_groups(legacy_logs):
        errors.append("equipment_check_logs: legacy day-wide equipment unique must be removed")

    gate_events = models.get("gate_events")
    if gate_events:
        gate_columns = columns(gate_events)
        identity = gate_columns.get("gate_event_id", {})
        if not identity.get("primary_key"):
            errors.append("gate_events.gate_event_id must be the append-only event identity")
        if "updated_at" in gate_columns:
            errors.append("gate_events must be append-only and cannot expose updated_at")

    if not MAPPING_PATH.is_file():
        errors.append(f"missing conversion map: {MAPPING_PATH.relative_to(ROOT)}")
    else:
        mapping = MAPPING_PATH.read_text().lower()
        for anchor in sorted(MAPPING_ANCHORS):
            if anchor.lower() not in mapping:
                errors.append(f"conversion map missing anchor: {anchor}")

    return errors


def main() -> int:
    errors = validate()
    if errors:
        print(f"schema-v2 contract: FAIL ({len(errors)} errors)")
        for error in errors:
            print(f"- {error}")
        return 1
    print("schema-v2 contract: PASS")
    print(f"validated {len(REQUIRED_TABLES)} required #29/#34 source models")
    print("validated work assignment, token hash, idempotency, claim, and gate-event identities")
    print("validated PR #48 conversion map anchors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
