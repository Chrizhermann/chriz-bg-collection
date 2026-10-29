"""October 29 intake: Artisan's Kitpack tweak 1010000 as a default (alpha.22).

Christopher, 2026-10-26 ("Ja, aufnehmen"): "Update EEex-based Weapon Usability Changes" sweeps
weapons that mods installed after the Kitpack add into the Kitpack's EEex-based weapon rules
(C0PR#CL/C0PR#KG). In the October 29 EET lab (curated Recommended) it opened IWDification's
two-handed axes to the Priest of Tempus and the Eldritch Knight and the SCS Drow Katana to the
Arcane Trickster sorcerer; Tweaks Anthology's two-handed axes were already Kitpack copies.

Its TP2 installs it only when C0PR#CL.SPL exists. In the pinned chriz.4 source the generator
that creates that file (eeex/profs_eeex.tpa) is included by Eldritch Knight 20000 with EEex
installed, by Arcane Trickster sorcerer thief equipment 8204 (which requires EEex), and by
Samurai 1010 and Favored Soul 30001, which the collection does not offer. The feature therefore
requires EEex and 20000 or 8204. It is the last component of the late tweak run, which follows
Tweaks Anthology and SCS; 1110 (Samurai items) stays unavailable with 1010.

Static recipe checks for both profiles only. Nothing here is a game installation.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import NATIVE_RUN_ORDERS, _effective_features, build_recipe
from tools.curation_audit import Decision, RowKey, load_catalogs, load_curation_map


ROOT = Path(__file__).resolve().parents[2]
KITPACK_EVIDENCE = "manifest/evidence/release-intake-20261027.toml"
RUN = "artisanskitpack-tweak-late-bg2"
SWEEP = "feature:artisanskitpack-tweak:component-1010000"
EEEX_CORE = "feature:eeex:mandatory-components"
ELDRITCH_KNIGHT = "feature:artisanskitpack:component-20000"
ARCANE_TRICKSTER = "feature:artisanskitpack-tweak:component-8204"
LATE_RUN = [51010, 12012, 3202, 3302, 8204, 1010000]


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = _load(directory / "collection.toml")
    preset = _load(directory / "presets/chris-recommended.toml")["selections"]
    return collection, preset


class Oct29KitpackWeaponSweepTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.directories = [ROOT / "manifest", ROOT / "recipes/curated-full-current", cls.output]
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        (cls.tweak_record,) = [record for record in _load(ROOT / KITPACK_EVIDENCE)["artifacts"]
                               if record["mod_id"] == "artisanskitpack-tweak"]

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def _each_profile(self):
        for name, (directory, collection, preset) in self.profiles.items():
            with self.subTest(profile=name):
                yield name, directory, collection, preset

    @staticmethod
    def _selected(collection, preset, **changes):
        state = _effective_features(collection, {**preset, **changes})
        return {
            (ref["run_id"], ref["component"])
            for feature in collection["features"]
            if state[feature["id"]]
            for ref in feature.get("components", [])
        }

    def test_the_sweep_is_the_last_late_tweak_in_native_order(self):
        native = self.tweak_record["native_components"]
        self.assertIn(1010000, native)
        self.assertEqual(NATIVE_RUN_ORDERS[RUN], LATE_RUN)
        self.assertEqual(LATE_RUN, [component for component in native if component in LATE_RUN])
        for directory in self.directories:
            with self.subTest(profile=str(directory)):
                mod = _load(directory / "mods/artisanskitpack-tweak.toml")
                names = {component["id"]: component["name"] for component in mod["components"]}
                self.assertEqual(names[1010000], "Update EEex-based Weapon Usability Changes")
                runs = _load(directory / "collection.toml")["runs"]
                (late,) = [run for run in runs if run["run_id"] == RUN]
                self.assertEqual(late["components"], LATE_RUN)
                order = [run["run_id"] for run in runs]
                # The lab examples come from SCS (both profiles) and IWDification (the
                # curated profile).
                for earlier in ("artisanskitpack-main-bg2", "sirene-bg2", "artisanskitpack-npc-bg2",
                                "stratagems-bg2"):
                    self.assertLess(order.index(earlier), order.index(RUN), earlier)
                if "cdtweaks-bg2" in order:
                    for earlier in ("cdtweaks-bg2", "iwdification-bg2", "randomiser-bg2"):
                        self.assertLess(order.index(earlier), order.index(RUN), earlier)
                self.assertLess(order.index(RUN), order.index("eet-end-bg2"))

    def test_default_with_eeex_and_an_eldritch_knight_or_arcane_trickster_source(self):
        for _, _, collection, preset in self._each_profile():
            feature = {feature["id"]: feature for feature in collection["features"]}[SWEEP]
            self.assertEqual((feature["category"], feature["decision"], feature["readiness"]),
                             ("kits", "default", "ready"))
            self.assertEqual(feature["parent"], "mod:artisanskitpack-tweak")
            self.assertEqual(feature["requires"], [EEEX_CORE])
            self.assertEqual(feature["requires_any"], [ELDRITCH_KNIGHT, ARCANE_TRICKSTER])
            self.assertEqual(feature.get("conflicts", []), [])
            self.assertEqual(feature["components"], [{"run_id": RUN, "component": 1010000}])
            for phrase in ("weapon rules", "after the Kitpack", "Drow Katana", "Arcane Trickster sorcerer"):
                self.assertIn(phrase, feature["description"])
            self.assertEqual(preset.get(SWEEP), "on")

            baseline = self._selected(collection, preset)
            self.assertIn((RUN, 1010000), baseline)
            for eldritch_knight, arcane_trickster, expected in (("on", "off", True), ("off", "on", True),
                                                                ("off", "off", False)):
                selected = self._selected(collection, preset,
                                          **{ELDRITCH_KNIGHT: eldritch_knight, ARCANE_TRICKSTER: arcane_trickster})
                self.assertIs((RUN, 1010000) in selected, expected, (eldritch_knight, arcane_trickster))
            self.assertNotIn((RUN, 1010000), self._selected(collection, preset, **{"mod:eeex": "off"}))
            self.assertEqual(baseline - self._selected(collection, preset, **{SWEEP: "off"}), {(RUN, 1010000)})

    def test_catalog_map_and_changelog_record_the_decision(self):
        rows = {row.component_id: row for row in load_catalogs(ROOT / "docs/curation/components")
                if row.catalog == "ARTISANSKITPACK_TWEAK"}
        self.assertEqual(rows[1010000].decision, Decision.DEFAULT)
        # Samurai item repair 1110 stays unavailable while Samurai 1010 is uncurated.
        self.assertEqual(rows[1110].decision, Decision.EXCLUDED)
        curation_map = load_curation_map(ROOT / "manifest/curation-map.toml")
        (catalog,) = [entry for entry in curation_map.catalogs if entry.catalog == "ARTISANSKITPACK_TWEAK"]
        self.assertNotIn(1010000, catalog.excluded)
        self.assertIn(1110, catalog.excluded)
        (target,) = [target for target in curation_map.targets if target.target_id == SWEEP]
        self.assertEqual(target.rows, (RowKey("ARTISANSKITPACK_TWEAK", 1010000),))

        text = (ROOT / "docs/curation/components/ARTISANSKITPACK_TWEAK.md").read_text(encoding="utf-8")
        for phrase in ("`C0PR#CL.SPL`", "Ja, aufnehmen", "`20000`", "`8204`"):
            self.assertIn(phrase, text)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        self.assertIn("**Kit weapon rules for weapons from later mods** — **Default.**", changelog)


if __name__ == "__main__":
    unittest.main()
