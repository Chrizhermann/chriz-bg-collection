"""October 23 intake: optional quest mods, and BG1 Unfinished Business made optional.

NTotSC at upstream master f0ef953, Ascalon's Questpack v7.0, Black Hearts v2.05, Thalantyr
Item Upgrade v4.5.1, the Icewind Dale Voice Pack 1.2.0 and PPE at DaftHunk fork commit
417226ea enter both recipe profiles unchecked in the Quest mods category. Static pin,
evidence, catalog, selection and order checks only; nothing here is a game installation or
gameplay acceptance.
"""

from pathlib import Path
import hashlib
import re
import subprocess
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import _effective_features, build_recipe
from tools.curation_audit import Decision, load_catalogs


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261023-quest-mods.toml"
NTOTSC_COMMIT = "f0ef95323630bc38df7dd4c969a2b48a71ebec82"
PPE_COMMIT = "417226ea6642b8c682b2d049d0ec6e86f38b7f47"
# Per mod: the pin, its run, the offered components in TP2 order, and whether a GitHub
# release (not only a commit) publishes the source.
PINS = {
    "ntotsc": {
        "artifact": f"ntotsc-{NTOTSC_COMMIT}",
        "kind": "github-commit-zip",
        "reference": NTOTSC_COMMIT,
        "filename": f"NTotSC-{NTOTSC_COMMIT}.zip",
        "run": "ntotsc-bg2",
        "offered": list(range(17)),
        "published_release": False,
    },
    "ac-quest": {
        "artifact": "ac-quest-7.0",
        "kind": "github-release",
        "reference": "v7.0",
        "filename": "ascalons-questpack-7.0.iemod",
        "run": "ac-quest-bg2",
        "offered": list(range(13)),
        "published_release": True,
    },
    "blackhearts": {
        "artifact": "blackhearts-2.05",
        "kind": "github-release",
        "reference": "v2.05",
        "filename": "BlackHearts-v2.05.iemod",
        "run": "blackhearts-bg2",
        "offered": [1],
        "published_release": True,
    },
    "thalan": {
        "artifact": "thalan-4.5.1",
        "kind": "github-release",
        "reference": "v4.5.1",
        "filename": "thalantyr-item-upgrade-v4.5.1.iemod",
        "run": "thalan-bg2",
        "offered": [0],
        "published_release": True,
    },
    "zg-iwd-voices": {
        "artifact": "zg-iwd-voices-1.2.0",
        "kind": "github-release",
        "reference": "version-1.2.0",
        "filename": "zg-iwd-voices-1.2.0.iemod",
        "run": "zg-iwd-voices-bg2",
        "offered": [0],
        "published_release": True,
    },
    "ppe": {
        "artifact": f"ppe-{PPE_COMMIT}",
        "kind": "github-commit-zip",
        "reference": PPE_COMMIT,
        "filename": f"Portraits-Portraits-Everywhere-{PPE_COMMIT}.zip",
        "run": "ppe-bg2",
        "offered": [0, 100, 150, 10, 20, 25, 300, 301, 302, 311, 312, 322, 330, 332, 340, 342, 350, 30, 40],
        "published_release": False,
    },
}
QUEST_RUNS = {pin["run"] for pin in PINS.values()}
# Only the main components and Ascalon's first variants start selected inside a ticked mod.
DEFAULTS = {("ntotsc-bg2", 0), ("ac-quest-bg2", 2), ("ac-quest-bg2", 5), ("ac-quest-bg2", 10), ("ppe-bg2", 0)}
# Catalog rows: everything offered is optional except the defaults; these are not offered.
EXCLUDED = {("BLACKHEARTS", 20005), ("PPE", 310), ("PPE", 320)}
CATALOGS = {"NTOTSC": "ntotsc-bg2", "AC_QUEST": "ac-quest-bg2", "BLACKHEARTS": "blackhearts-bg2",
            "THALAN": "thalan-bg2", "ZG_IWD_VOICES": "zg-iwd-voices-bg2", "PPE": "ppe-bg2"}
BG1UB_COMPONENTS = [0, 11, 12, 13, 14, 16, 17, 18, 19, 21, 22, 29, 30, 32, 33, 34]
# (run, options, default option) for every either-or choice.
CHOICES = [
    ("ntotsc-bg2", [13, 14], None),
    ("ntotsc-bg2", [15, 16], None),
    ("ac-quest-bg2", [2, 3], 2),
    ("ac-quest-bg2", [5, 6], 5),
    ("ac-quest-bg2", [10, 11], 10),
    ("ppe-bg2", [100, 150], None),
    ("ppe-bg2", [300, 301, 302], None),
    ("ppe-bg2", [311, 312], None),
    ("ppe-bg2", [330, 332], None),
    ("ppe-bg2", [340, 342], None),
]
BLACKHEARTS = "feature:blackhearts:component-1"
SOD_REMIX_CORE = "feature:chriz-sod-remix:mandatory-components"
# Twelve further mods are on hold until their authors consent, and nothing about them may
# appear in a tracked file, so this test names them only by digest: the first 16 hex digits
# of the SHA-256 of an identifier's words in lower case, joined without separators. The
# identifiers are each mod's id, archive id, TP2 or repository name, title, and its catalog,
# mod-file and feature names.
ON_HOLD = frozenset({
    "03ed760260a2d23f", "094b55fd6c6816c7", "0d19dfbe9902e6cb", "1065f984b1c65485", "145aa0063d1c8624",
    "1e4e27af536b1ca7", "231f281c8e24d101", "2c78183e0585f1ef", "3e2a059dfd9ac476", "3f86d44e0f668921",
    "42c32f9610c6ffff", "4e0f263976fa7923", "503ea6c561c59168", "7463ca44f7eb5407", "892a8de3680557c2",
    "8d6c80dcdb9e8f12", "8e723247772a8046", "92e30cc7acdfad6f", "94e9604fabc7ddaa", "9dd891a00db37331",
    "a5d6cb5a5c92efb5", "b3830de49cfc54cd", "b63b475e2197234f", "be927d9695f74e52", "c9683873db75a01d",
    "cd8a78210cd57790", "db838a02d7bff7f9", "dd4574be03129a56", "ee3f7fc8f4e07f5b", "ff49039549466501",
})
# Digests of each identifier's first word, so only plausible starts are expanded.
ON_HOLD_STARTS = frozenset({
    "03ed760260a2d23f", "1065f984b1c65485", "231f281c8e24d101", "2ad5623197671570", "3c482346f3750276",
    "42e234b7e83423c5", "4e0f263976fa7923", "695073cb6649c0a4", "754a7b453d97b648", "77fbd5b243a7a898",
    "7b12be43febc6e18", "7edb360f06acaef2", "892a8de3680557c2", "8a1eb106427026fd", "8d6c80dcdb9e8f12",
    "8fb6d5f37e8055ce", "ab8b76517c436cdf", "ad2015263f339bff", "b3830de49cfc54cd", "bdf4888a34a4c042",
    "be927d9695f74e52", "dd4574be03129a56", "e6bb5689beec52c4", "e88559cc13f4588e",
})
ON_HOLD_MAX_WORDS = 5


def _digest(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()[:16]


def _on_hold_hits(text: str, starts: dict[str, bool]) -> list[str]:
    words = re.findall(r"[a-z0-9]+", text.lower())
    found = []
    for index, word in enumerate(words):
        if word not in starts:
            starts[word] = _digest(word) in ON_HOLD_STARTS
        if starts[word]:
            for count in range(1, ON_HOLD_MAX_WORDS + 1):
                candidate = "".join(words[index:index + count])
                if _digest(candidate) in ON_HOLD:
                    found.append(candidate)
    return found


def _load_profile(directory: Path) -> tuple[dict, dict]:
    collection = tomllib.loads((directory / "collection.toml").read_text(encoding="utf-8"))
    preset = tomllib.loads((directory / "presets/chris-recommended.toml").read_text(encoding="utf-8"))["selections"]
    return collection, preset


def _feature(run_id: str, component: int) -> str:
    return f"feature:{run_id.removesuffix('-bg2')}:component-{component}"


class Oct23QuestModsIntakeTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.output = Path(cls.temp.name) / "recipe"
        # Synthetic evidence marker used only in this temporary test output.
        build_recipe(ROOT, cls.output, "0" * 40)
        cls.profiles = {
            "curated": (cls.output, *_load_profile(cls.output)),
            "fallback": (ROOT / "manifest", *_load_profile(ROOT / "manifest")),
        }
        cls.evidence = tomllib.loads((ROOT / EVIDENCE).read_text(encoding="utf-8"))
        cls.records = {record["mod_id"]: record for record in cls.evidence["artifacts"]}

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

    @staticmethod
    def _run(selected, run_id):
        return {component for selected_run, component in selected if selected_run == run_id}

    def test_intake_evidence_matches_current_pins(self):
        self.assertEqual(self.evidence["verified_on"], "2026-10-23")
        self.assertEqual(set(self.records), set(PINS))
        self.assertEqual(len(self.evidence["artifacts"]), len(PINS))
        for mod_id, pin in PINS.items():
            record = self.records[mod_id]
            with self.subTest(mod=mod_id):
                self.assertEqual(record["artifact_id"], pin["artifact"])
                self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
                self.assertEqual(record["source_reference"], pin["reference"])
                for flag in ("fresh_download", "fresh_extraction", "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)
                self.assertIs(record["published_release_verified"], pin["published_release"])
                for digest in ("actual_sha256", "tp2_sha256"):
                    self.assertRegex(record[digest], r"^[0-9a-f]{64}$")
                self.assertRegex(record["source_commit"], r"^[0-9a-f]{40}$")
                # The run installs the offered components in the TP2's declaration order.
                native = record["native_components"]
                self.assertEqual([component for component in native if component in pin["offered"]], pin["offered"])
        for mod_id, commit in (("ntotsc", NTOTSC_COMMIT), ("ppe", PPE_COMMIT)):
            self.assertEqual(self.records[mod_id]["source_commit"], commit)
            self.assertIn(commit, self.records[mod_id]["immutable_source_url"])
        # Only Black Hearts 20005 and PPE 310/320 are declared but not offered.
        self.assertEqual(
            {(mod_id, component) for mod_id, pin in PINS.items()
             for component in self.records[mod_id]["native_components"] if component not in pin["offered"]},
            {("blackhearts", 20005), ("ppe", 310), ("ppe", 320)},
        )

    def test_both_profiles_pin_the_verified_archives(self):
        for profile in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.output):
            for mod_id, pin in PINS.items():
                record = self.records[mod_id]
                with self.subTest(profile=str(profile), mod=mod_id):
                    mod = tomllib.loads((profile / f"mods/{mod_id}.toml").read_text(encoding="utf-8"))
                    self.assertEqual(mod["artifact_id"], pin["artifact"])
                    self.assertEqual([component["id"] for component in mod["components"]], pin["offered"])
                    artifact = tomllib.loads(
                        (profile / f"artifacts/{pin['artifact']}.toml").read_text(encoding="utf-8"))
                    for key, measured in (("sha256", "actual_sha256"), ("expected_length", "actual_length"),
                                          ("url", "immutable_source_url"), ("reference", "source_reference")):
                        self.assertEqual(artifact["source"][key], record[measured])
                    self.assertEqual(artifact["source"]["kind"], pin["kind"])
                    self.assertEqual(artifact["source"]["expected_filename"], pin["filename"])
                    self.assertEqual(artifact["acquisition"], "fetch-only")
                    self.assertEqual(artifact["archive"]["publish_roots"], record["published_roots"])
        # The checked-in curated files are the generator's copies of the authored ones.
        for mod_id, pin in PINS.items():
            for name in (f"artifacts/{pin['artifact']}.toml", f"mods/{mod_id}.toml"):
                self.assertEqual((ROOT / "recipes/curated-full-current" / name).read_bytes(),
                                 (ROOT / "manifest" / name).read_bytes())

    def test_catalogs_offer_every_component_except_the_three_that_cannot_install(self):
        rows, bg1ub = {}, {}
        for row in load_catalogs(ROOT / "docs/curation/components"):
            if row.catalog in CATALOGS:
                rows[(row.catalog, row.component_id)] = row.decision
            elif row.catalog == "BG1UB" and row.decision != Decision.EXCLUDED:
                bg1ub[row.component_id] = row.decision
        # Every offered BG1UB row is optional now, as for UB in BG2.
        self.assertEqual(bg1ub, {component: Decision.OPTIONAL for component in BG1UB_COMPONENTS})
        expected = {}
        for catalog, run_id in CATALOGS.items():
            mod_id = run_id.removesuffix("-bg2")
            for component in PINS[mod_id]["offered"]:
                expected[(catalog, component)] = (
                    Decision.DEFAULT if (run_id, component) in DEFAULTS else Decision.OPTIONAL)
        for key in EXCLUDED:
            expected[key] = Decision.EXCLUDED
        self.assertEqual(rows, expected)

    def test_quest_mods_and_bg1ub_stay_out_of_recommended_and_the_default_selection(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            quest = [feature for feature in collection["features"] if feature["category"] == "quest-mods"]
            self.assertEqual(
                {(ref["run_id"]) for feature in quest for ref in feature.get("components", [])}, QUEST_RUNS)
            for feature in quest:
                self.assertEqual(feature["readiness"], "experimental", feature["id"])
                self.assertIn(feature["decision"], {"optional", "default"}, feature["id"])
            self.assertEqual(
                {(ref["run_id"], ref["component"]) for feature in quest if feature["decision"] == "default"
                 for ref in feature["components"]},
                DEFAULTS,
            )
            for parent in ("mod:ntotsc", "mod:ac-quest", "mod:ppe", "mod:bg1ub"):
                self.assertEqual(features[parent]["decision"], "optional")
                self.assertNotEqual(preset.get(parent), "on")
            for feature in collection["features"]:
                if feature.get("parent") == "mod:bg1ub":
                    self.assertEqual(feature["decision"], "optional", feature["id"])
            # The engine's default selection follows the decisions without a preset.
            decided = {feature["id"]: "on" if feature["decision"] in {"default", "mandatory"} else "off"
                       for feature in collection["features"]}
            for selection in (preset, decided):
                selected = self._selected(collection, selection)
                self.assertFalse({run for run, _ in selected} & (QUEST_RUNS | {"bg1ub-bg1"}), name)
            # Ticking a mod alone adds only its decided defaults.
            baseline = self._selected(collection, preset)
            for switch, run_id, expected in (("mod:ntotsc", "ntotsc-bg2", {0}),
                                              ("mod:ac-quest", "ac-quest-bg2", {2, 5, 10}),
                                              ("mod:ppe", "ppe-bg2", {0}),
                                              ("feature:thalan:component-0", "thalan-bg2", {0}),
                                              ("feature:zg-iwd-voices:component-0", "zg-iwd-voices-bg2", {0})):
                ticked = self._selected(collection, preset, **{switch: "on"})
                self.assertEqual(ticked - baseline, {(run_id, component) for component in expected}, switch)
                self.assertEqual(baseline - ticked, set(), switch)
            # Ticking BG1UB and its 16 components plans all of them and nothing else.
            bg1ub = {"mod:bg1ub": "on", **{f"feature:bg1ub:component-{c}": "on" for c in BG1UB_COMPONENTS}}
            ticked = self._selected(collection, preset, **bg1ub)
            self.assertEqual(ticked - baseline, {("bg1ub-bg1", c) for c in BG1UB_COMPONENTS})
            self.assertEqual(baseline - ticked, set())

    def test_every_choice_resolves_to_exactly_one_side(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            groups = {}
            for feature in collection["features"]:
                if feature["category"] == "quest-mods" and feature.get("choice_group"):
                    groups.setdefault(feature["choice_group"], []).append(feature["id"])
            self.assertEqual(
                sorted(groups.values()),
                sorted([_feature(run_id, option) for option in options] for run_id, options, _ in CHOICES),
            )
            for run_id, options, default in CHOICES:
                ids = [_feature(run_id, option) for option in options]
                for feature_id in ids:
                    self.assertEqual({conflict["feature_id"] for conflict in features[feature_id]["conflicts"]},
                                     set(ids) - {feature_id})
                parent = features[ids[0]]["parent"]
                # Component 0 stays on: NTotSC 13/14 and PPE 100/150 require it.
                base = {parent: "on", _feature(run_id, 0): "on"}
                chosen = self._run(self._selected(collection, preset, **base), run_id) & set(options)
                self.assertEqual(chosen, set() if default is None else {default}, (name, ids))
                for option in options:
                    picks = {_feature(run_id, other): "on" if other == option else "off" for other in options}
                    chosen = self._run(self._selected(collection, preset, **base, **picks), run_id) & set(options)
                    self.assertEqual(chosen, {option}, (name, option))
                # Requesting every side at once never plans two of them.
                every = {feature_id: "on" for feature_id in ids}
                chosen = self._run(self._selected(collection, preset, **base, **every), run_id) & set(options)
                self.assertLessEqual(len(chosen), 1, (name, ids))

    def test_ntotsc_never_plans_nim_or_svlast_with_the_bg1ub_encounters(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            for quest, encounter in ((4, 30), (6, 32)):
                quest_id, encounter_id = f"feature:ntotsc:component-{quest}", f"feature:bg1ub:component-{encounter}"
                conflict, = features[quest_id]["conflicts"]
                self.assertEqual(conflict["feature_id"], encounter_id)
                title = features[encounter_id]["title"]
                self.assertIn(f"untick {title} under BG1 Unfinished Business", conflict["reason"])
                self.assertEqual(features[encounter_id].get("conflicts", []), [])
                for bg1ub_on in (False, True):
                    for encounter_on in (False, True):
                        selected = self._selected(collection, preset, **{
                            "mod:ntotsc": "on", "feature:ntotsc:component-0": "on", quest_id: "on",
                            "mod:bg1ub": "on" if bg1ub_on else "off",
                            encounter_id: "on" if encounter_on else "off",
                        })
                        has_quest = quest in self._run(selected, "ntotsc-bg2")
                        has_encounter = encounter in self._run(selected, "bg1ub-bg1")
                        self.assertFalse(has_quest and has_encounter, (name, quest, encounter))
                        # BG1UB's encounter is preselected and wins whenever it is selected.
                        self.assertEqual(has_encounter, bg1ub_on and encounter_on)
                        self.assertEqual(has_quest, not (bg1ub_on and encounter_on))

    def test_ppe_310_and_320_are_never_planned(self):
        for name, _, collection, preset in self._each_profile():
            ppe_run = next(run for run in collection["runs"] if run["run_id"] == "ppe-bg2")
            self.assertFalse({310, 320} & set(ppe_run["components"]))
            for feature in collection["features"]:
                for ref in feature.get("components", []):
                    self.assertFalse(ref["run_id"] == "ppe-bg2" and ref["component"] in {310, 320}, feature["id"])
            everything = {feature["id"]: "on" for feature in collection["features"]
                          if feature.get("parent") == "mod:ppe" and not feature.get("choice_group")}
            for pick in range(3):
                choices = {}
                for run_id, options, _ in CHOICES:
                    if run_id == "ppe-bg2":
                        chosen = options[min(pick, len(options) - 1)]
                        choices.update({_feature(run_id, option): "on" if option == chosen else "off"
                                        for option in options})
                planned = self._run(self._selected(collection, preset, **{"mod:ppe": "on"}, **everything, **choices),
                                    "ppe-bg2")
                self.assertFalse({310, 320} & planned, name)
                self.assertEqual(len(planned), 13, (name, sorted(planned)))
            # Category portraits need the core component.
            planned = self._run(self._selected(collection, preset, **{
                "mod:ppe": "on", "feature:ppe:component-0": "off", "feature:ppe:component-100": "on"}), "ppe-bg2")
            self.assertNotIn(100, planned)

    def test_black_hearts_is_unavailable_while_the_sod_remix_is_selected(self):
        for name, _, collection, preset in self._each_profile():
            features = {feature["id"]: feature for feature in collection["features"]}
            conflict, = features[BLACKHEARTS]["conflicts"]
            self.assertEqual(conflict["feature_id"], SOD_REMIX_CORE)
            self.assertTrue(conflict["reason"].startswith("Unavailable with CHRIZ-SOD-REMIX"))
            self.assertEqual(features[SOD_REMIX_CORE]["decision"], "mandatory")
            selected = self._selected(collection, preset, **{BLACKHEARTS: "on"})
            self.assertEqual(self._run(selected, "blackhearts-bg2"), set(), name)
            self.assertTrue(self._run(selected, "chriz-sod-remix-bg2"))
            # The declaration is the only blocker: without the Remix, Black Hearts resolves.
            selected = self._selected(collection, preset, **{BLACKHEARTS: "on", "mod:chriz-sod-remix": "off"})
            self.assertEqual(self._run(selected, "blackhearts-bg2"), {1}, name)
            self.assertEqual(self._run(selected, "chriz-sod-remix-bg2"), set())

    def test_runs_keep_the_reviewed_order_in_the_recipe_and_the_resolved_plan(self):
        everything = {"mod:ntotsc": "on", "mod:ac-quest": "on", "mod:ppe": "on", "feature:thalan:component-0": "on",
                      "feature:zg-iwd-voices:component-0": "on", BLACKHEARTS: "on",
                      "mod:bg1ub": "on", "feature:bg1ub:component-0": "on"}
        for name, _, collection, preset in self._each_profile():
            recipe = [run["run_id"] for run in collection["runs"]]
            phases = {run["run_id"]: run["phase"] for run in collection["runs"]}
            orders = {"recipe": recipe}
            for label, changes in (("plan", everything), ("plan without the Remix",
                                                          {**everything, "mod:chriz-sod-remix": "off"})):
                selected = {run for run, _ in self._selected(collection, preset, **changes)}
                orders[label] = [run for run in recipe if run in selected]
            for label, order in orders.items():
                with self.subTest(order=label):
                    at = order.index
                    self.assertEqual("blackhearts-bg2" in order, label != "plan")
                    self.assertTrue({"ntotsc-bg2", "ac-quest-bg2", "thalan-bg2", "zg-iwd-voices-bg2",
                                     "ppe-bg2"} <= set(order))
                    for run_id in QUEST_RUNS & set(order):
                        self.assertEqual(phases[run_id], "main", run_id)
                        self.assertLess(at(run_id), at("eet-end-bg2"), run_id)
                    # NTotSC and Ascalon's follow EET and both fixpacks and precede the NPC mods.
                    self.assertLess(at("eet-initialize-bg2"), at("ntotsc-bg2"))
                    self.assertLess(at("eefixpack-bg2"), at("ntotsc-bg2"))
                    self.assertLess(at("ntotsc-bg2"), at("ac-quest-bg2"))
                    self.assertLess(at("ac-quest-bg2"), at("rr-bg2"))
                    if "blackhearts-bg2" in order:
                        self.assertLess(at("ac-quest-bg2"), at("blackhearts-bg2"))
                        self.assertLess(at("blackhearts-bg2"), at("rr-bg2"))
                    # The IWD Voice Pack refuses an unmerged sod-dlc.zip.
                    self.assertLess(at("dlcmerger-bg1"), at("zg-iwd-voices-bg2"))
                    # Thalantyr after NTotSC; before the Item Randomiser, SCS and the Tweaks Anthology.
                    self.assertLess(at("ntotsc-bg2"), at("thalan-bg2"))
                    for later in ("randomiser-bg2", "stratagems-bg2", "cdtweaks-bg2"):
                        if later in order:
                            self.assertLess(at("thalan-bg2"), at(later))
                            self.assertLess(at("ac-quest-bg2"), at(later))
                    # PPE after the NPC mods, SCS, the Item Randomiser and the companion
                    # conversions, directly before continuity 199, which precedes EET_end.
                    for earlier in ("randomiser-bg2", "stratagems-bg2", "chriz-bg-modpack-pre-continuity-bg2"):
                        self.assertLess(at(earlier), at("ppe-bg2"))
                    self.assertEqual(at("ppe-bg2") + 1, at("chriz-bg-modpack-continuity-bg2"))
                    self.assertEqual(at("chriz-bg-modpack-continuity-bg2") + 1, at("eet-end-bg2"))
            main_after_ppe = [run for run in recipe[recipe.index("ppe-bg2") + 1:] if phases[run] == "main"]
            self.assertEqual(main_after_ppe, ["chriz-bg-modpack-continuity-bg2"], name)

    def test_on_hold_digests_are_well_formed_and_spare_the_offered_mods(self):
        self.assertEqual(len(ON_HOLD), 30)
        self.assertEqual(len(ON_HOLD_STARTS), 24)
        self.assertTrue(all(re.fullmatch(r"[0-9a-f]{16}", value) for value in ON_HOLD | ON_HOLD_STARTS))
        starts = {}
        for mod_id, pin in PINS.items():
            text = " ".join([mod_id, pin["artifact"], pin["filename"], pin["run"], f"mod:{mod_id}"])
            self.assertEqual(_on_hold_hits(text, starts), [], mod_id)

    def test_no_on_hold_mod_appears_in_any_tracked_file(self):
        listed = subprocess.run(["git", "ls-files", "-z"], cwd=ROOT, check=True, capture_output=True).stdout
        paths = [path for path in listed.decode("utf-8").split("\0") if path]
        self.assertGreater(len(paths), 500)
        starts = {}
        hits = []
        for path in paths:
            hits += [(path, hit) for hit in _on_hold_hits(path, starts)]
            file = ROOT / path
            if not file.is_file():
                continue
            data = file.read_bytes()
            if b"\0" in data:
                continue
            for number, line in enumerate(data.decode("utf-8", errors="replace").splitlines(), 1):
                hits += [(f"{path}:{number}", hit) for hit in _on_hold_hits(line, starts)]
        self.assertEqual(hits, [])


if __name__ == "__main__":
    unittest.main()
