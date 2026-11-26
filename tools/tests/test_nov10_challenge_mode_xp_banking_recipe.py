"""November 10: Challenge Mode includes campaign XP caps with chapter caps (Christopher, 2026-11-10).

Chriz BG Modpack 650 (campaign XP caps with banked XP) and 652 (chapter-by-chapter XP caps)
join the Challenge Mode group in both Chriz Easy BG profiles. 652 requires 650; the engine turns
both on with one switch. 651 (the Shadows of Amn cap) conflicts with 652 and stays a separate
optional choice outside the group. Challenge Mode stays off in Chris's Recommended, so its
selection and component count do not change. Chriz Easy IWD has no such group.
Christopher, 2026-11-22: no travel back into an earlier campaign (Modpack 680) also joins the
group, after 652, and is off in Chris's Recommended.
Static recipe checks only; nothing here is a game installation.
"""

from pathlib import Path
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features


ROOT = Path(__file__).resolve().parents[2]
GROUP = "challenge-mode"
RUN = "chriz-bg-modpack-campaign-xp-bg2"
CAMPAIGN_CAPS = "feature:chriz-bg-modpack:component-650"
SOA_CAP = "feature:chriz-bg-modpack:component-651"
CHAPTER_CAPS = "feature:chriz-bg-modpack:component-652"
WORLD_PROGRESSION = "feature:chriz-bg-modpack:component-680"
WORLD_PROGRESSION_RUN = "chriz-bg-modpack-bg2"
# Recommended component counts (curated: 466 in the 0.1.2 lab + Chriz Overhead Text 0 on Nov 16).
# November 18 adds Round Clock 361. Modpack 680 is a Challenge Mode member and off, so it adds none.
# November 22 adds the independent Chriz Multiplayer Fix defaults 0 and 1 to both profiles.
# November 23 adds Modpack alpha.18 default 470 to both profiles.
# November 27 adds Modpack alpha.19 default 695 to both profiles.
# November 27 adds Modpack alpha.20 defaults 697 and 698 to both profiles.
PROFILES = {"manifest": 379, "recipes/curated-full-current": 474}


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _selected(collection: dict, selection: dict) -> set[tuple[str, int]]:
    effective = _effective_features(collection, selection)
    return {(ref["run_id"], ref["component"]) for feature in collection["features"]
            if effective[feature["id"]] for ref in feature.get("components", [])}


class Nov10ChallengeModeXpBankingTests(unittest.TestCase):
    def _profiles(self):
        for directory, count in PROFILES.items():
            collection = _load(ROOT / directory / "collection.toml")
            preset = _load(ROOT / directory / "presets/chris-recommended.toml")["selections"]
            (group,) = [group for group in collection["feature_groups"] if group["id"] == GROUP]
            yield directory, count, collection, preset, group

    def test_both_profiles_add_650_and_652_but_not_651(self):
        for directory, _, collection, _, group in self._profiles():
            with self.subTest(profile=directory):
                members = group["members"]
                self.assertIn(CAMPAIGN_CAPS, members)
                self.assertIn(CHAPTER_CAPS, members)
                self.assertNotIn(SOA_CAP, members)
                # Directly after the authored members of October 13, before the curated-only one.
                self.assertEqual(members.index(CAMPAIGN_CAPS), 4)
                self.assertEqual(members.index(CHAPTER_CAPS), 5)
                # Christopher, 2026-11-22: Modpack 680 follows the XP caps.
                self.assertEqual(members.index(WORLD_PROGRESSION), 6)
                features = {feature["id"]: feature for feature in collection["features"]}
                self.assertEqual(features[CHAPTER_CAPS]["requires"], [CAMPAIGN_CAPS])
                self.assertEqual([item["feature_id"] for item in features[SOA_CAP]["conflicts"]],
                                 [CHAPTER_CAPS])
                for feature_id in (CAMPAIGN_CAPS, SOA_CAP, CHAPTER_CAPS):
                    self.assertEqual(features[feature_id]["decision"], "optional", feature_id)
                    self.assertEqual(features[feature_id]["readiness"], "experimental", feature_id)
                    self.assertIn("(experimental)", features[feature_id]["title"], feature_id)

    def test_switching_the_group_on_installs_650_and_652(self):
        for directory, _, collection, preset, group in self._profiles():
            with self.subTest(profile=directory):
                baseline = _selected(collection, preset)
                on = _selected(collection, {**preset, **{member: "on" for member in group["members"]}})
                self.assertEqual({item for item in on - baseline if item[0] == RUN},
                                 {(RUN, 650), (RUN, 652)})
                self.assertNotIn((RUN, 651), on)
                self.assertEqual(baseline - on, set())
                self.assertIn((WORLD_PROGRESSION_RUN, 680), on - baseline)

    def test_recommended_keeps_challenge_mode_and_xp_caps_off(self):
        for directory, count, collection, preset, group in self._profiles():
            with self.subTest(profile=directory):
                self.assertNotIn(GROUP, preset)
                for member in group["members"]:
                    self.assertNotEqual(preset.get(member, "off"), "on", member)
                for feature_id in (CAMPAIGN_CAPS, SOA_CAP, CHAPTER_CAPS, WORLD_PROGRESSION):
                    self.assertNotIn(feature_id, preset)
                selected = _selected(collection, preset)
                self.assertFalse(any(run == RUN for run, _ in selected))
                self.assertNotIn((WORLD_PROGRESSION_RUN, 680), selected)
                self.assertEqual(len(selected), count)

    def test_chriz_easy_iwd_has_no_challenge_mode_group(self):
        collection = _load(ROOT / "recipes/iwd-current/collection.toml")
        self.assertEqual(collection.get("feature_groups", []), [])
        ids = {feature["id"] for feature in collection["features"]}
        self.assertFalse({CAMPAIGN_CAPS, SOA_CAP, CHAPTER_CAPS, WORLD_PROGRESSION} & ids)


if __name__ == "__main__":
    unittest.main()
