from __future__ import annotations

import tomllib
import unittest

from tools.recipe_presentation import FeaturePresentation, apply_feature_presentations


LIGHTNING = """\
[[features]]
id = "feature:spell-rev:component-80"
title = "Classic bouncing Lightning Bolt"
description = "Keep the approved spell behavior."
choice_group = "spell-rev-lightning-bolt"
decision = "default"
readiness = "ready"
parent = "mod:spell-rev"
requires = ["feature:spell-rev:mandatory-components"]
components = [{ run_id = "spell-rev-lightning-bg2", component = 80 }]
"""


class FeaturePresentationTests(unittest.TestCase):
    def test_existing_lightning_choice_is_written_once(self) -> None:
        presentation = FeaturePresentation(
            "Spell Revisions", "Lightning Bolt behavior", "spell-rev-lightning-bolt"
        )
        result = apply_feature_presentations(
            LIGHTNING, {"feature:spell-rev:component-80": presentation}
        )
        feature = tomllib.loads(result)["features"][0]
        self.assertEqual(result.count('choice_group = '), 1)
        self.assertEqual(feature["choice_group"], "spell-rev-lightning-bolt")
        for key, value in tomllib.loads(LIGHTNING)["features"][0].items():
            self.assertEqual(feature[key], value, key)

    def test_existing_presentation_fields_are_updated_idempotently(self) -> None:
        source = LIGHTNING.replace(
            'decision = "default"',
            'source_label = "Old source"\ngroup_label = "Old group"\ndecision = "default"',
        )
        presentations = {
            "feature:spell-rev:component-80": FeaturePresentation(
                "Spell Revisions", "Lightning Bolt behavior", "spell-rev-lightning-bolt"
            )
        }
        result = apply_feature_presentations(source, presentations)
        feature = tomllib.loads(result)["features"][0]
        self.assertEqual(feature["source_label"], "Spell Revisions")
        self.assertEqual(feature["group_label"], "Lightning Bolt behavior")
        self.assertEqual(apply_feature_presentations(result, presentations), result)

    def test_unmapped_presentation_preserves_the_authored_choice(self) -> None:
        result = apply_feature_presentations(
            LIGHTNING,
            {"feature:spell-rev:component-80": FeaturePresentation("Spell Revisions", "Spells")},
        )
        self.assertEqual(
            tomllib.loads(result)["features"][0]["choice_group"], "spell-rev-lightning-bolt"
        )


if __name__ == "__main__":
    unittest.main()
