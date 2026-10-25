"""Shared fixtures establish language-boundary compatibility, including failures."""

import json
from pathlib import Path
import unittest
from pydantic import ValidationError
from apss_session_capture.inventory import (
    CaptureReceipt,
    QualifiedTranscript,
    operation_adapter,
)


class InventoryConformance(unittest.TestCase):
    def test_capture_receipts(self):
        path = Path(__file__).resolve().parents[2] / "fixtures/capture-receipts.json"
        for case in json.loads(path.read_text()):
            with self.subTest(case=case["name"]):
                identity = QualifiedTranscript.model_validate(case["identity"])
                try:
                    receipt = CaptureReceipt.model_validate_json(
                        json.dumps(case["receipt"])
                    )
                except ValidationError:
                    accepted = False
                else:
                    accepted = receipt.validates_capture(identity, case["content_hash"])
                    self.assertEqual(
                        json.loads(receipt.model_dump_json()), case["receipt"]
                    )
                self.assertEqual(accepted, case["accepted"])

    def test_qualified_storage_keys(self):
        path = (
            Path(__file__).resolve().parents[2] / "fixtures/qualified-storage-keys.json"
        )
        keys = set()
        for case in json.loads(path.read_text()):
            identity = QualifiedTranscript.model_validate(case["identity"])
            self.assertEqual(identity.storage_key(), case["storage_key"])
            self.assertNotIn(identity.storage_key(), keys)
            keys.add(identity.storage_key())

    def test_shared_contract(self):
        path = (
            Path(__file__).resolve().parents[2] / "fixtures/inventory-conformance.json"
        )
        for case in json.loads(path.read_text()):
            with self.subTest(case=case["name"]):
                raw = json.dumps(case["value"], ensure_ascii=False)
                if case["valid"]:
                    parsed = operation_adapter.validate_json(raw)
                    self.assertEqual(
                        json.loads(parsed.model_dump_json()), case["value"]
                    )
                else:
                    with self.assertRaises(ValidationError):
                        operation_adapter.validate_json(raw)

    def test_schema_is_current(self):
        from apss_session_capture.schema import inventory_schema

        path = (
            Path(__file__).resolve().parents[2]
            / "schemas/inventory-operation.schema.json"
        )
        self.assertEqual(path.read_text(), inventory_schema())
