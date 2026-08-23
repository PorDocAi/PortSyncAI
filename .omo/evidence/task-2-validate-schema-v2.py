#!/usr/bin/env python3
"""Validate the additive schema-v2 source contract for issues #29 and #34."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path
from typing import Any, TypeAlias


JsonObject: TypeAlias = dict[str, Any]

DEFAULT_ROOT = Path(__file__).resolve().parents[2]
ROOT = Path(os.environ.get("PORTDOC_SCHEMA_ROOT", DEFAULT_ROOT)).resolve()
MODELS_DIR = ROOT / "apis/api/models"
MIGRATIONS_DIR = ROOT / "apis/api/migrations"
MAPPING_PATH = ROOT / ".omo/evidence/task-2-pr48-conversion-map.md"

IMMUTABLE_LEGACY_TABLES = {
    "attendances",
    "cargo_documents",
    "cargo_items",
    "equipment",
    "equipment_check_logs",
    "gate_terminals",
    "gate_verify_logs",
    "work_assignments",
}

REQUIRED_TABLES = {
    "containers",
    "container_cargo_items",
    "cargo_document_versions",
    "cargo_item_documents",
    "work_types",
    "works",
    "work_targets",
    "v2_work_assignments",
    "education_courses",
    "education_target_rules",
    "education_completions",
    "ppe_requirements",
    "work_ppe_requirement_snapshots",
    "equipment_profiles",
    "equipment_tag_tokens",
    "work_assignment_equipment",
    "shared_equipment_claims",
    "equipment_check_events",
    "work_preparations",
    "work_stops",
    "gate_events",
    *IMMUTABLE_LEGACY_TABLES,
}

REQUIRED_COLUMNS = {
    "container_cargo_items": {"container_id", "cargo_item_id"},
    "cargo_document_versions": {
        "cargo_document_version_id",
        "legacy_cargo_document_id",
        "document_type",
        "document_version",
        "processing_status",
        "review_status",
        "reviewed_by_id",
        "reviewed_at",
    },
    "cargo_item_documents": {
        "cargo_item_id",
        "cargo_document_version_id",
        "document_role",
    },
    "works": {"work_type_id", "status", "scheduled_start_at", "scheduled_end_at"},
    "work_targets": {"work_id", "target_type", "container_id", "cargo_item_id"},
    "v2_work_assignments": {
        "work_assignment_id",
        "legacy_assignment_id",
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
        "source_document_version_id",
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
        "source_document_version_id",
        "source_document_version",
        "reviewed_by_id",
        "snapshotted_at",
    },
    "equipment_profiles": {
        "equipment_profile_id",
        "legacy_equipment_id",
        "equipment_type_id",
        "ownership_type",
        "owner_employee_id",
        "status",
        "manufacturer_replacement_due_at",
    },
    "equipment_tag_tokens": {
        "equipment_profile_id",
        "tag_token_hash",
        "is_active",
        "issued_by_id",
        "issued_at",
        "deactivated_by_id",
        "deactivated_at",
    },
    "work_assignment_equipment": {
        "work_assignment_id",
        "equipment_profile_id",
        "requirement_snapshot_id",
        "accepted_at",
    },
    "shared_equipment_claims": {
        "work_assignment_id",
        "equipment_profile_id",
        "claimed_at",
    },
    "equipment_check_events": {
        "employee_id",
        "work_assignment_id",
        "equipment_profile_id",
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
    ("cargo_document_versions", "document_type"): {"BL", "DGD", "CI", "MSDS"},
    ("equipment_profiles", "ownership_type"): {"PERSONAL", "SHARED"},
    ("equipment_profiles", "status"): {
        "AVAILABLE",
        "BLOCKED",
        "DAMAGED",
        "LOST",
        "REPLACED",
    },
    ("works", "status"): {"PLANNED", "ACTIVE", "STOPPED", "COMPLETED", "CANCELLED"},
    ("v2_work_assignments", "status"): {
        "ASSIGNED",
        "SELECTED",
        "ACTIVE",
        "COMPLETED",
        "CANCELLED",
    },
    ("gate_events", "decision"): {"PASS", "BLOCK"},
}

REQUIRED_UNIQUES = {
    "cargo_item_documents": [
        (
            "uq_cargo_item_document_role",
            {"cargo_item_id", "cargo_document_version_id", "document_role"},
        ),
    ],
    "v2_work_assignments": [
        ("uq_v2_work_assignment_work_employee", {"work_id", "employee_id"}),
    ],
    "equipment_check_events": [
        ("uq_equipment_check_employee_idempotency", {"employee_id", "idempotency_key"}),
    ],
    "work_assignment_equipment": [
        ("uq_assignment_equipment", {"work_assignment_id", "equipment_profile_id"}),
    ],
    "shared_equipment_claims": [
        ("uq_shared_equipment_active_claim", {"equipment_profile_id"}),
    ],
    "gate_events": [
        ("uq_gate_event_terminal_idempotency", {"terminal_id", "idempotency_key"}),
    ],
}

REQUIRED_FOREIGN_KEYS = {
    ("cargo_document_versions", "legacy_cargo_document_id"): (
        "cargo_documents.cargo_document_id"
    ),
    ("cargo_item_documents", "cargo_document_version_id"): (
        "cargo_document_versions.cargo_document_version_id"
    ),
    ("v2_work_assignments", "legacy_assignment_id"): (
        "work_assignments.assignment_id"
    ),
    ("equipment_profiles", "legacy_equipment_id"): "equipment.equipment_id",
    ("equipment_tag_tokens", "equipment_profile_id"): (
        "equipment_profiles.equipment_profile_id"
    ),
    ("work_assignment_equipment", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("work_assignment_equipment", "equipment_profile_id"): (
        "equipment_profiles.equipment_profile_id"
    ),
    ("shared_equipment_claims", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("shared_equipment_claims", "equipment_profile_id"): (
        "equipment_profiles.equipment_profile_id"
    ),
    ("equipment_check_events", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("equipment_check_events", "equipment_profile_id"): (
        "equipment_profiles.equipment_profile_id"
    ),
    ("work_preparations", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("work_stops", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("work_stops", "legacy_attendance_id"): "attendances.attendance_id",
    ("gate_events", "work_assignment_id"): (
        "v2_work_assignments.work_assignment_id"
    ),
    ("gate_events", "legacy_verify_log_id"): (
        "gate_verify_logs.verify_log_id"
    ),
    ("gate_events", "legacy_attendance_id"): "attendances.attendance_id",
    ("ppe_requirements", "source_document_version_id"): (
        "cargo_document_versions.cargo_document_version_id"
    ),
    ("work_ppe_requirement_snapshots", "source_document_version_id"): (
        "cargo_document_versions.cargo_document_version_id"
    ),
}

NULLABLE_LEGACY_BRIDGES = {
    ("cargo_document_versions", "legacy_cargo_document_id"),
    ("v2_work_assignments", "legacy_assignment_id"),
    ("equipment_profiles", "legacy_equipment_id"),
    ("work_stops", "legacy_attendance_id"),
    ("gate_events", "legacy_verify_log_id"),
    ("gate_events", "legacy_attendance_id"),
}

MAPPING_ANCHORS = {
    "immutable legacy physical tables",
    "cargo_document_versions",
    "v2_work_assignments",
    "equipment_profiles",
    "legacy_cargo_document_id",
    "legacy_assignment_id",
    "legacy_equipment_id",
    "legacy_attendance_id",
    "legacy_verify_log_id",
    "gate_verify_logs",
    "cargo_documents.review_status",
    "attendances.work_status",
    "equipment_check_logs",
    "equipment.nfc_tag_uid",
    "source",
    "target",
    "conversion",
    "retained legacy",
    "v2 authoritative",
}


def load_models() -> tuple[dict[str, JsonObject], list[str]]:
    models: dict[str, JsonObject] = {}
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


def load_historical_tables() -> tuple[dict[str, JsonObject], list[str]]:
    tables: dict[str, JsonObject] = {}
    errors: list[str] = []
    paths = sorted(MIGRATIONS_DIR.glob("000[1-4]_*.vespertide.json"))
    if len(paths) != 4:
        errors.append(f"expected historical migrations 0001-0004, found {len(paths)}")
        return tables, errors

    for path in paths:
        try:
            migration = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError) as error:
            errors.append(f"{path.name}: invalid JSON: {error}")
            continue
        for action in migration.get("actions", []):
            action_type = action.get("type")
            table_name = action.get("table")
            if action_type == "create_table" and isinstance(table_name, str):
                tables[table_name] = {
                    "columns": list(action.get("columns", [])),
                    "constraints": list(action.get("constraints", [])),
                }
            elif action_type == "add_column" and isinstance(table_name, str):
                if table_name not in tables:
                    errors.append(f"{path.name}: add_column references unknown table {table_name}")
                    continue
                tables[table_name]["columns"].append(action.get("column", {}))
    return tables, errors


def columns(model: JsonObject) -> dict[str, JsonObject]:
    return {
        column["name"]: column
        for column in model.get("columns", [])
        if isinstance(column, dict) and isinstance(column.get("name"), str)
    }


def unique_groups(model: JsonObject) -> dict[str, set[str]]:
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


def foreign_key_reference(column: JsonObject) -> str | None:
    foreign_key = column.get("foreign_key")
    if isinstance(foreign_key, dict):
        reference = foreign_key.get("references")
        return reference if isinstance(reference, str) else None
    return foreign_key if isinstance(foreign_key, str) else None


def validate_immutable_legacy_tables(
    models: dict[str, JsonObject], historical: dict[str, JsonObject]
) -> list[str]:
    errors: list[str] = []
    for table_name in sorted(IMMUTABLE_LEGACY_TABLES):
        model = models.get(table_name)
        expected = historical.get(table_name)
        if model is None or expected is None:
            if expected is None:
                errors.append(f"historical schema missing immutable table: {table_name}")
            continue

        actual_columns = columns(model)
        expected_columns = columns(expected)
        missing = expected_columns.keys() - actual_columns.keys()
        extra = actual_columns.keys() - expected_columns.keys()
        for column_name in sorted(missing):
            errors.append(f"{table_name}: missing historical column {column_name}")
        for column_name in sorted(extra):
            errors.append(f"{table_name}: v2 column must move to an additive table: {column_name}")
        for column_name in sorted(expected_columns.keys() & actual_columns.keys()):
            if actual_columns[column_name] != expected_columns[column_name]:
                errors.append(
                    f"{table_name}.{column_name}: definition differs from migrations 0001-0004"
                )

        actual_constraints = model.get("constraints", [])
        expected_constraints = expected.get("constraints", [])
        if actual_constraints != expected_constraints:
            errors.append(
                f"{table_name}: constraints differ from migrations 0001-0004"
            )
    return errors


def validate() -> list[str]:
    models, errors = load_models()
    historical, historical_errors = load_historical_tables()
    errors.extend(historical_errors)

    for table in sorted(REQUIRED_TABLES - models.keys()):
        errors.append(f"missing model: {table}")

    errors.extend(validate_immutable_legacy_tables(models, historical))

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
                errors.append(f"{table}: unique {name} must cover {sorted(expected_columns)}")

    for (table, column_name), expected_reference in REQUIRED_FOREIGN_KEYS.items():
        if table not in models:
            continue
        column = columns(models[table]).get(column_name)
        if column is None:
            continue
        actual_reference = foreign_key_reference(column)
        if actual_reference != expected_reference:
            errors.append(
                f"{table}.{column_name}: FK must reference {expected_reference}, "
                f"found {actual_reference}"
            )

    for table, column_name in sorted(NULLABLE_LEGACY_BRIDGES):
        if table not in models:
            continue
        column = columns(models[table]).get(column_name)
        if column is not None and column.get("nullable") is not True:
            errors.append(f"{table}.{column_name}: legacy bridge must be nullable")

    versions = models.get("cargo_document_versions")
    if versions:
        legacy_document = columns(versions).get("legacy_cargo_document_id", {})
        if legacy_document.get("unique") is not True:
            errors.append(
                "cargo_document_versions.legacy_cargo_document_id must uniquely identify an imported legacy row"
            )

    profiles = models.get("equipment_profiles")
    if profiles:
        legacy_equipment = columns(profiles).get("legacy_equipment_id", {})
        if legacy_equipment.get("unique") is not True:
            errors.append(
                "equipment_profiles.legacy_equipment_id must uniquely identify an imported legacy asset"
            )

    assignments = models.get("v2_work_assignments")
    if assignments:
        legacy_assignment = columns(assignments).get("legacy_assignment_id", {})
        if legacy_assignment.get("unique") is not True:
            errors.append(
                "v2_work_assignments.legacy_assignment_id must uniquely identify an imported legacy assignment"
            )

    tokens = models.get("equipment_tag_tokens")
    if tokens:
        token_hash = columns(tokens).get("tag_token_hash", {})
        if token_hash.get("type") != {"kind": "char", "length": 64}:
            errors.append("equipment_tag_tokens.tag_token_hash must be a 64-character hash")
        if token_hash.get("unique") is not True:
            errors.append("equipment_tag_tokens.tag_token_hash must be unique")

    for table_name in ("equipment_check_events", "gate_events"):
        model = models.get(table_name)
        if model and "updated_at" in columns(model):
            errors.append(f"{table_name}: append-only events cannot expose updated_at")

    gate_events = models.get("gate_events")
    if gate_events:
        identity = columns(gate_events).get("gate_event_id", {})
        if not identity.get("primary_key"):
            errors.append("gate_events.gate_event_id must be the append-only event identity")

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
    print(
        f"validated {len(IMMUTABLE_LEGACY_TABLES)} immutable legacy tables "
        "against migrations 0001-0004"
    )
    print(f"validated {len(REQUIRED_TABLES)} required legacy/v2 source models")
    print("validated additive legacy-ID bridges and v2 authoritative relationships")
    print("validated token, allocation, claim, check-event, preparation, stop, and gate contracts")
    print("validated PR #48 conversion map anchors")
    return 0


if __name__ == "__main__":
    sys.exit(main())
