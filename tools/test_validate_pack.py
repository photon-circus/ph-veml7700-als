"""Focused regression tests for pack safety policy."""

import unittest

from tools.pack_policy import (
    automation_contains_publish_command,
    publication_is_hard_disabled,
)


class PublicationGuardTests(unittest.TestCase):
    def test_only_explicit_false_is_accepted(self) -> None:
        self.assertTrue(publication_is_hard_disabled({"package": {"publish": False}}))
        self.assertFalse(publication_is_hard_disabled({"package": {"publish": True}}))
        self.assertFalse(publication_is_hard_disabled({"package": {}}))

    def test_publish_commands_are_detected_in_automation(self) -> None:
        self.assertTrue(automation_contains_publish_command("cargo publish --dry-run"))
        self.assertTrue(automation_contains_publish_command("${{ secrets.CRATES_IO_TOKEN }}"))
        self.assertTrue(automation_contains_publish_command("env: CARGO_REGISTRY_TOKEN"))
        self.assertFalse(automation_contains_publish_command("cargo package --list"))


if __name__ == "__main__":
    unittest.main()
