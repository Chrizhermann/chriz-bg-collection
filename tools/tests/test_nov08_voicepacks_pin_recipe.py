"""November 8 intake: voicepacks release v1.2.0 replaces the v1.1.0 package pins.

The 18 player soundsets keep their component-0 runs, order and Recommended selection.  The
release writes each selected registration regardless of the existing charsnd.2da data; audio is
unchanged.  This is source/package and owner fixture evidence, not collection-install or gameplay
acceptance.
"""

from pathlib import Path
import tempfile
import tomllib
import unittest

from tools.curated_full_recipe import build_recipe


ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = "manifest/evidence/release-intake-20261108-voicepacks-chriz-easy-bg.toml"
RELEASE_COMMIT = "0afae24b1467d141fca6007b72992fca12ec529e"
STATIC_COMMIT = "8995d051d21af6f5beedcad791ee91ffa515b2e2"
# (mod id suffix, folder, artifact version, length, SHA-256, TP2 SHA-256)
PACKS = (
    ("neera", "cd_neera", "1.0.2", 2_363_513, "b7d029f839062b5e7763de61f541a4922e2cb73d82b6e6c035d8fcdcff3eac94", "0d2063ca6200556812da39638aa048f3a56367072071510bebd898385a580e25"),
    ("safana-bg1", "cd_safana", "1.0.2", 2_949_451, "daf45b3d58901509a05f570269e417ad137dcd920cc4cbfc14fbeafe6272f276", "234fce68c535f24c9ca68cfc17eb01a290a9bbe3ab481194157994577a2b4ce7"),
    ("safana-sod", "cd_safsod", "1.0.2", 2_751_308, "eacccb9187a1abaa28d8d8e6171e3437459f1280b04bec9c9c4199e7f33a2be2", "14dd9125ee0a40dab6dcf3cd94dc6b63c2cc0d4c2b74d8a385db34ab6f744cee"),
    ("jaheira-bg1", "cd_jahebg1", "1.0.2", 2_712_742, "7167f278446bc9d63485aebf54cefe281d52d6659a496a2c070417fc734322e8", "2317d3accaf5bd6fa7813c07a0e5e38a1db5a91d73e8aad43995f5e71cf0c686"),
    ("jaheira-bg2", "cd_jahebg2", "1.0.2", 2_121_640, "1c629463c66b0cc377e35356e560e1d26868eab61124fa704c26d61843df8bcf", "c9bce446fb8fbb3c7428e2cb1bfa7cb6e227bb707c42d99fad5bf90f239a0198"),
    ("yeslick", "cd_yeslick", "1.0.2", 2_713_135, "b4118fecf98786bd092056f9039ee6d8591d5419aa09cdb5b8807ba85bd2c7d6", "09333374a0a68601d34dcebacb2ca2e56dd19c6142b26e5f62e7107bb2b1ebe1"),
    ("edwin-bg1", "cd_edwnbg1", "1.0.2", 2_906_433, "478b58972a4f247e00c046e02ab6c2c87f3fb520d3b10f037b730ba855061a5b", "e0a8e2d5de5c268758931c012aadbfdc166dbcc2ee27143f8011a8b2740a4ce0"),
    ("edwin-bg2", "cd_edwnbg2", "1.0.2", 2_457_030, "131d4db31326c377ddfde2f73009d1bc18e8d62ff0f053d66389d6be684b8e3d", "5510fd043a70ae24585ffc3836cad6c74b5783947e06155cc65b33d4eb7b942a"),
    ("aerie", "cd_aerie", "1.0.2", 2_343_511, "a6c0d61b87803df550dd97cdd9e56e443ee268a06e5751e9b6b4cb2cbba93f18", "39aa683825b8486f96b1ad8f1ed75847047ed8fc5f018907af910157660ced2f"),
    ("imoen-bg1", "cd_imobg1", "1.0.2", 1_798_679, "7bac3a24dbd5ecdc70afb2b82210c2e6e39061fcb4f2b7803aeb9c911452d098", "82380c5b130daa42d7e9bc023eb3d716073f5cdedcf7e2a8cf43fd406dad51f2"),
    ("imoen-bg2", "cd_imobg2", "1.0.2", 2_048_827, "98125ee8e3159ec7c968649e769fa1de0073c0d9dc8c41675c62247586ebe3d7", "dd90c6eb85984959a41c5097fb1ee1732789a2a65109c5a4611d40b63ff2f7be"),
    ("sarevok", "cd_sarevok", "1.0.2", 2_117_790, "29589c43eb14191124155b246979270c137af2f1303a70e8df245173d1c31882", "5a62baed5e3e7c03384fb338aa397bfa0d62d404fd56b71667ffd6afdfb45d65"),
    ("amelyssan", "cd_amely", "1.0.2", 2_450_051, "d16ed2b2b773b5e6caf1b67597eab4c1717ea0014ddbf90e969cc1d808b2693a", "b0c439310e698c3517db91180341670f46575c8b12da18e60ae8e167277f01e2"),
    ("gromnir-il-khan", "cd_gromn", "1.0.2", 1_974_964, "ce709ab49aea5db3e6ac1419995301d66c64c9bf13e79f55be1968926a363e0b", "ebc8f35af3e232e0ad4692de03cfd0b01a3dbd0f97e69de358147e5c7e8d6e15"),
    ("ajantis", "cd_ajanti", "2.0.1", 4_059_707, "1c9bb4056e8266b2868a223c22fe9675407763bf8c2796e96a4c1590dc6e73f3", "7fef8bebc552ff68e806c919c628384aceb7107ce486ed01b739629d089e037b"),
    ("viconia-bg1", "cd_vicbg1", "1.0.1", 4_122_902, "a1f9b57cd841861cc87eec665b862c215089ae51548994b1fc193f45904708f6", "d495018f81f4a8a738b7a0182da49e7d0907f4b6f44d4d1811b64b8dc7f85390"),
    ("viconia-bg2", "cd_vicbg2", "1.0.1", 5_060_194, "68fbfca869d7d4a5ef795a7b2d8573b886448dd724da873a24d53ced1b74b56f", "14f7c8d68f6dafa873e64fe9cf17e2d86718370fe727bd1d3c1c97b96705f4e8"),
    ("garithos", "cd_garith", "1.1.1", 5_119_782, "84d153f3e7608243ab57b32a3f57d1f34301b602a0196566da48fa7ce646f2a8", "07d5bf2c846a4e77b2be895fdacd2b95e287198c16492d0f50e398c3c9b8d6fe"),
)
CATALOGS = {
    "aerie": "CD_AERIE.md", "ajantis": "CD_AJANTI.md", "amelyssan": "CD_AMELY.md",
    "edwin-bg1": "CD_EDWNBG1.md", "edwin-bg2": "CD_EDWNBG2.md", "garithos": "CD_GARITH.md",
    "gromnir-il-khan": "CD_GROMN.md", "imoen-bg1": "CD_IMOBG1.md", "imoen-bg2": "CD_IMOBG2.md",
    "jaheira-bg1": "CD_JAHEBG1.md", "jaheira-bg2": "CD_JAHEBG2.md", "neera": "CD_NEERA.md",
    "safana-bg1": "CD_SAFANA.md", "safana-sod": "CD_SAFSOD.md", "sarevok": "CD_SAREVOK.md",
    "viconia-bg1": "CD_VICBG1.md", "viconia-bg2": "CD_VICBG2.md", "yeslick": "CD_YESLICK.md",
}


def load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class Nov08VoicepacksPinTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temp = tempfile.TemporaryDirectory()
        cls.generated = Path(cls.temp.name) / "recipe"
        build_recipe(ROOT, cls.generated, "0" * 40)
        cls.evidence = load(ROOT / EVIDENCE)

    @classmethod
    def tearDownClass(cls):
        cls.temp.cleanup()

    def test_intake_evidence_freezes_all_v120_release_assets(self):
        self.assertEqual(self.evidence["verified_on"], "2026-11-08")
        for phrase in ("fresh downloads", "14 generated packs", "old 30-row table", "missing table",
                       "four hand-built packs", "not a combined collection install or gameplay acceptance"):
            self.assertIn(phrase, self.evidence["scope"])
        records = self.evidence["artifacts"]
        self.assertEqual(len(records), len(PACKS))
        for record, (pack, folder, version, length, sha256, tp2_sha256) in zip(records, PACKS):
            with self.subTest(pack=pack):
                self.assertEqual(record["mod_id"], f"voicepack-{pack}")
                self.assertEqual(record["artifact_id"], f"voicepack-{pack}-{version}")
                self.assertEqual(record["recipe_profiles"], ["manifest", "recipes/curated-full-current"])
                self.assertEqual(record["source_reference"], "v1.2.0")
                self.assertEqual(record["source_commit"], RELEASE_COMMIT)
                self.assertEqual((record["actual_length"], record["actual_sha256"]), (length, sha256))
                self.assertEqual(record["tp2_path"], f"{folder}/{folder}.tp2")
                self.assertEqual(record["tp2_sha256"], tp2_sha256)
                self.assertEqual(record["published_roots"], [folder])
                self.assertEqual(record["native_components"], [0])
                for flag in ("fresh_download", "fresh_extraction", "published_release_verified",
                             "native_component_order_verified"):
                    self.assertIs(record[flag], True, flag)

    def test_both_profiles_and_generated_recipe_pin_the_evidence(self):
        records = {record["mod_id"]: record for record in self.evidence["artifacts"]}
        for directory in (ROOT / "manifest", ROOT / "recipes/curated-full-current", self.generated):
            for pack, folder, version, length, sha256, _ in PACKS:
                with self.subTest(profile=directory, pack=pack):
                    mod_id = f"voicepack-{pack}"
                    record = records[mod_id]
                    mod = load(directory / f"mods/{mod_id}.toml")
                    artifact = load(directory / f"artifacts/{record['artifact_id']}.toml")
                    self.assertEqual(mod["artifact_id"], record["artifact_id"])
                    self.assertEqual(mod["tp2"], record["tp2_path"])
                    self.assertEqual([component["id"] for component in mod["components"]], [0])
                    self.assertEqual(artifact["source"]["reference"], "v1.2.0")
                    self.assertEqual(artifact["source"]["expected_length"], length)
                    self.assertEqual(artifact["source"]["sha256"], sha256)

    def test_static_records_catalogs_changelog_and_pin_list_name_the_release(self):
        run_ids = {f"voicepack-{pack}-bg2" for pack, *_ in PACKS}
        for relative in (
            "manifest/releases/v0.1.0-alpha.1/acceptance.toml",
            "recipes/curated-full-current/releases/v0.1.0-alpha.1/acceptance.toml",
        ):
            records = {item["subject_id"]: item for item in load(ROOT / relative)["evidence"]}
            for run_id in run_ids:
                with self.subTest(record=relative, run=run_id):
                    record = records[run_id]
                    self.assertEqual(record["commit"], STATIC_COMMIT)
                    self.assertEqual(record["test_artifact"], "tools/tests/test_nov08_voicepacks_pin_recipe.py")
                    self.assertEqual(record["date"], "2026-11-08")
        for pack, _, version, *_ in PACKS:
            catalog = (ROOT / "docs/curation/components" / CATALOGS[pack]).read_text(encoding="utf-8")
            self.assertIn(f"Listed at release version **{version}** (ChrizFader's voicepacks v1.2.0).", catalog)
            self.assertIn("writes its selected registration regardless of what `charsnd.2da` holds", catalog)
        changelog = (ROOT / "docs/plans/2026-10-14-next-release-changelogs.md").read_text(encoding="utf-8")
        draft = changelog[changelog.index("## Collection 0.1.2 (draft)"):changelog.index("## Installer changelog")]
        for phrase in ("**Voicepacks write their registrations as decided**", "v1.2.0",
                       "audio is unchanged"):
            self.assertIn(phrase, draft)
        pin_list = (ROOT / "docs/pin-list-2.7.md").read_text(encoding="utf-8")
        for pack, _, version, _, _, _ in PACKS:
            self.assertEqual(pin_list.count(
                f"/v1.2.0/{pack}-player-voicepack-v{version}.zip"), 2)


if __name__ == "__main__":
    unittest.main()
