"""Consume separately approved literal bytes and independent invalid/bound inputs."""

import copy
import hashlib
import json
import unittest

from project_semantics_support import (
    byte_fixture,
    byte_vector,
    csv_mutation,
    csv_rows,
    rehash_ascii_core,
)

from arboresce_contract_validation.project_exports import (
    INAPPLICABLE,
    ExportReadError,
    canonical_bytes,
    read_cell,
    read_export,
    read_field_tree,
    read_json_tree,
    verify_artifact,
)


class ExportBytesTests(unittest.TestCase):
    def test_four_reviewed_complete_byte_vectors(self):
        for name in (
            "project-json-v1",
            "project-csv-v1",
            "project-financial-json-v1",
            "project-financial-csv-v1",
        ):
            with self.subTest(format=name):
                artifact, data, expected = byte_vector(name)
                self.assertEqual(read_export(data, name), expected)
                self.assertEqual(verify_artifact(artifact, data), expected)

    def test_unknown_literal_vector(self):
        with self.assertRaises(StopIteration):
            byte_vector("unregistered")

    def test_reviewed_encoding_tree_and_depths(self):
        for name in ("encoding-tree", "encoding-depth-32"):
            expected = json.loads(byte_fixture(name + ".json"))
            self.assertEqual(read_json_tree(byte_fixture(name + ".json")), expected)
            self.assertEqual(read_field_tree(byte_fixture(name + ".fields.csv")), expected)
            self.assertEqual(canonical_bytes(expected), byte_fixture(name + ".json"))
        for suffix, reader in ((".json", read_json_tree), (".fields.csv", read_field_tree)):
            with self.assertRaisesRegex(ExportReadError, "depth"):
                reader(byte_fixture("encoding-depth-33" + suffix))

    def test_reviewed_cell_grammar(self):
        cases = json.loads(byte_fixture("encoding-cells.json"))
        for case in cases["valid"]:
            with self.subTest(token=case["token"]):
                self.assertEqual(read_cell(case["token"]), case["value"])
        self.assertIs(read_cell(cases["inapplicable"]), INAPPLICABLE)
        for token in cases["invalid"]:
            with self.subTest(invalid=token), self.assertRaises(ExportReadError):
                read_cell(token)

    def test_cell_published_unit_and_byte_bounds(self):
        self.assertEqual(read_cell("text:s:" + "a" * 32_753), "a" * 32_753)
        self.assertEqual(read_cell("text:s:" + "汉" * 21_843), "汉" * 21_843)
        for token in ("text:s:" + "a" * 32_754, "text:s:" + "汉" * 21_843 + "a"):
            with self.assertRaisesRegex(ExportReadError, "cell bound"):
                read_cell(token)
        for token in ("text:s:\ud800", "text:s:\\u000", "text:s:\\u00AG"):
            with self.assertRaises(ExportReadError):
                read_cell(token)

    def test_json_invalid_raw_bytes_and_numbers(self):
        inputs = (
            b"\xef\xbb\xbfnull",
            b"\xff",
            b"",
            b"{",
            b"null\n",
            b" null",
            b'{"b":0,"a":0}',
            b'{"a":0,"a":1}',
            b'{"a":0,"\\u0061":1}',
            b"-0",
            b"-1",
            b"1.0",
            b"1e0",
            b"NaN",
            b"Infinity",
            b"9007199254740992",
            b'"\\ud800"',
            b'{"\\ud800":null}',
            b'"\\u0041"',
            b"[" * 2_000 + b"null" + b"]" * 2_000,
        )
        for data in inputs:
            with self.subTest(data=data[:40]), self.assertRaises(ExportReadError):
                read_json_tree(data)
        for value in (-1, 1.5, "\ud800"):
            with self.assertRaises(ExportReadError):
                canonical_bytes(value)

    def test_json_actual_stream_boundary(self):
        data = b'"' + b"a" * 67_108_862 + b'"'
        self.assertEqual(len(read_json_tree(data)), 67_108_862)
        with self.assertRaisesRegex(ExportReadError, "stream bound"):
            read_json_tree(data + b" ")
        with self.assertRaisesRegex(ExportReadError, "canonical stream bound"):
            canonical_bytes("a" * 67_108_863)

    def test_json_actual_node_boundary(self):
        data = b"[" + b"null," * 1_048_574 + b"null]"
        self.assertEqual(len(read_json_tree(data)), 1_048_575)
        with self.assertRaisesRegex(ExportReadError, "node bound"):
            read_json_tree(b"[" + b"null," * 1_048_575 + b"null]")

    def test_csv_framing_and_header(self):
        data = byte_fixture("encoding-tree.fields.csv")
        invalid = (
            data[:-2],
            data.replace(b"\r\n", b"\n"),
            data.replace(b'"path"', b"path", 1),
            data.replace(b'"path"', b'"other"', 1),
            b'"path","value_type"\r\n',
            b'"path","value_type","value"\r\n"unterminated',
            b'"path","value_type","value"\r\n"x"junk,"x","x"\r\n',
            b'"path","value_type","value"\r\n',
            b"\r\n",
            b'"wrong","wrong","wrong"\r\n',
        )
        for value in invalid:
            with self.subTest(data=value[:40]), self.assertRaises(ExportReadError):
                read_field_tree(value)

    def test_field_tree_invalid_counts_types_paths_and_order(self):
        header = ["path", "value_type", "value"]
        root = ["text:s:", "text:s:object", "text:i:1"]
        scalar = ["text:s:/a", "text:s:null", "text:n:"]
        invalid = (
            [],
            [["text:s:/bad", "text:s:null", "text:n:"]],
            [["text:s:", "text:s:array", "text:b:true"]],
            [["text:s:", "text:s:array", "text:i:1048577"]],
            [root],
            [root, ["text:i:0", "text:s:null", "text:n:"]],
            [root, ["text:s:bad", "text:s:null", "text:n:"]],
            [root, ["text:s:/a/b", "text:s:null", "text:n:"]],
            [root, ["text:s:/~2", "text:s:null", "text:n:"]],
            [["text:s:", "text:s:object", "text:i:2"], scalar, scalar],
            [
                ["text:s:", "text:s:object", "text:i:2"],
                ["text:s:/b", "text:s:null", "text:n:"],
                scalar,
            ],
            [
                ["text:s:", "text:s:object", "text:i:2"],
                ["text:s:/a", "text:s:array", "text:i:1"],
                ["text:s:/a/0", "text:s:null", "text:n:"],
            ],
            [["text:s:", "text:s:null", "text:x:"]],
            [["text:s:", "text:s:boolean", "text:i:1"]],
            [["text:s:", "text:s:integer", "text:b:true"]],
            [["text:s:", "text:s:string", "text:n:"]],
            [["text:s:", "text:s:unknown", "text:n:"]],
            [["text:s:", "text:s:null", "text:n:"], scalar],
        )
        for rows in invalid:
            with self.subTest(rows=rows), self.assertRaises(ExportReadError):
                read_field_tree(csv_mutation([header, *rows]))

    def test_field_scalar_types(self):
        for kind, token, expected in (
            ("null", "text:n:", None),
            ("boolean", "text:b:false", False),
            ("integer", "text:i:0", 0),
            ("string", "text:s:", ""),
        ):
            data = csv_mutation(
                [
                    ["path", "value_type", "value"],
                    ["text:s:", "text:s:" + kind, token],
                ]
            )
            self.assertEqual(read_field_tree(data), expected)

    def test_csv_actual_node_overflow(self):
        data = (
            b'"path","value_type","value"\r\n'
            b'"text:s:","text:s:array","text:i:1048576"\r\n'
            + b"".join(
                b'"text:s:/' + str(n).encode() + b'","text:s:null","text:n:"\r\n'
                for n in range(1_048_576)
            )
        )
        self.assertLess(len(data), 67_108_864)
        with self.assertRaisesRegex(ExportReadError, "node bound"):
            read_field_tree(data)

    def test_payload_unknown_format_nonobject_and_closed_shape(self):
        for data, name in (
            (b"null", "project-json-v1"),
            (b"{}", "project-json-v1"),
            (b"{}", "unknown"),
        ):
            with self.assertRaises(ExportReadError):
                read_export(data, name)
        data = csv_mutation(
            [
                csv_rows(byte_fixture("project.csv"))[0],
                [
                    "text:s:field",
                    *("text:x:" for _ in range(5)),
                    "text:s:",
                    "text:s:null",
                    "text:n:",
                ],
            ]
        )
        with self.assertRaisesRegex(ExportReadError, "object"):
            read_export(data, "project-csv-v1")

    def test_project_csv_snapshot_and_profile_mutations(self):
        original = csv_rows(byte_fixture("project.csv"))
        for row, column, token in (
            (1, 8, "text:s:wrong"),
            (1, 6, "text:n:"),
            (2, 1, "text:s:99999999-9999-4999-8999-999999999999"),
        ):
            rows = copy.deepcopy(original)
            rows[row][column] = token
            with self.assertRaises(ExportReadError):
                read_export(csv_mutation(rows), "project-csv-v1")
        with self.assertRaises(ExportReadError):
            read_export(csv_mutation([original[0], *original[2:], original[1]]), "project-csv-v1")

    def test_financial_csv_overview_identity_and_complete_fields(self):
        original = csv_rows(byte_fixture("project-financial.csv"))
        for row, column, token in (
            (1, 23, "text:s:wrong"),
            (2, 9, "text:s:100.00"),
            (11, 1, "text:s:wrong"),
            (11, 9, "text:n:"),
        ):
            rows = copy.deepcopy(original)
            rows[row][column] = token
            with self.assertRaises(ExportReadError):
                read_export(csv_mutation(rows), "project-financial-csv-v1")
        rows = copy.deepcopy(original)
        rows[2], rows[3] = rows[3], rows[2]
        with self.assertRaises(ExportReadError):
            read_export(csv_mutation(rows), "project-financial-csv-v1")

    def test_artifact_closed_shape_core_scope_and_digest(self):
        artifact, data, _ = byte_vector("project-json-v1")
        mutations = (
            ("core", "extra"),
            ("reference", "export_id"),
            ("control", "subject", "domain_id"),
            ("core", "profile", "organization_id"),
            ("control", "last_event", "organization_id"),
            ("reference", "artifact_record_sha256"),
        )
        for path in mutations:
            candidate = copy.deepcopy(artifact)
            target = candidate
            for key in path[:-1]:
                target = target[key]
            target[path[-1]] = (
                "0" * 64 if "sha256" in path[-1] else "99999999-9999-4999-8999-999999999999"
            )
            with self.subTest(path=path), self.assertRaises(ExportReadError):
                verify_artifact(candidate, data)

    def test_financial_overview_boolean_integer_substitutions(self):
        artifact, data, _ = byte_vector("project-financial-csv-v1")
        original = csv_rows(data)
        index = next(n for n, row in enumerate(original) if row[0] == "text:s:summary")
        for column, token in (("complete", "text:i:1"), ("unknown_record_count", "text:b:false")):
            with self.subTest(column=column):
                rows = copy.deepcopy(original)
                rows[index][original[0].index(column)] = token
                changed = csv_mutation(rows)
                with self.assertRaisesRegex(ExportReadError, "overview"):
                    read_export(changed, "project-financial-csv-v1")
                candidate = copy.deepcopy(artifact)
                candidate["core"]["emitted"] = {
                    "sha256": hashlib.sha256(changed).hexdigest(),
                    "byte_length": len(changed),
                }
                rehash_ascii_core(candidate)
                with self.assertRaisesRegex(ExportReadError, "overview"):
                    verify_artifact(candidate, changed)

    def test_artifact_availability_exact_observation_and_byte_identity(self):
        artifact, data, _ = byte_vector("project-json-v1")
        for observed in ("2026-10-03T12:05:00.000Z", "2026-10-03T12:10:01.000Z"):
            candidate = copy.deepcopy(artifact)
            candidate["observed_at"] = observed
            with self.assertRaisesRegex(ExportReadError, "unavailable"):
                verify_artifact(candidate, data)
        candidate = copy.deepcopy(artifact)
        candidate["availability"] = "expired"
        with self.assertRaises(ExportReadError):
            verify_artifact(candidate, data)
        for emitted in (data + b" ", data.replace(b"project", b"projecX", 1)):
            with self.assertRaisesRegex(ExportReadError, "emitted byte identity"):
                verify_artifact(artifact, emitted)
        candidate = copy.deepcopy(artifact)
        candidate["core"]["payload_sha256"] = "0" * 64
        rehash_ascii_core(candidate)
        with self.assertRaisesRegex(ExportReadError, "payload digest"):
            verify_artifact(candidate, data)

    def test_artifact_payload_subject_joins(self):
        for name, paths in (
            (
                "project-json-v1",
                (
                    ("project", "project_id"),
                    ("snapshot_id",),
                    ("observed_control_revision", "value"),
                    ("project", "content_revision", "value"),
                ),
            ),
            (
                "project-financial-json-v1",
                (
                    ("report", "report_id"),
                    ("report", "domain_id"),
                ),
            ),
        ):
            artifact, data, _ = byte_vector(name)
            for path in paths:
                candidate = copy.deepcopy(artifact)
                target = candidate["core"]["subject"]
                for key in path[:-1]:
                    target = target[key]
                target[path[-1]] = (
                    "99" if path[-1] == "value" else "99999999-9999-4999-8999-999999999999"
                )
                rehash_ascii_core(candidate)
                with (
                    self.subTest(name=name, path=path),
                    self.assertRaisesRegex(ExportReadError, "payload subject"),
                ):
                    verify_artifact(candidate, data)


if __name__ == "__main__":
    unittest.main()
