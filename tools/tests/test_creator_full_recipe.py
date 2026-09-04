import tempfile
import unittest
from pathlib import Path

from tools.creator_full_recipe import (
    CURRENT_ARTISAN_NPC_COMPONENTS,
    CURRENT_MODPACK_COMPONENTS,
    CURRENT_SIRENE_COMPONENTS,
    Entry,
    build_ordered_runs,
    parse_weidu_log,
    private_publish_roots,
)


class CreatorFullRecipeTests(unittest.TestCase):
    def test_parse_weidu_log_preserves_path_component_and_name(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            log = Path(directory, "WeiDU.log")
            log.write_text(
                "// heading\n"
                "~MOD\\SETUP-MOD.TP2~ #0 #42 // Choice -> Display name: v1\n",
                encoding="utf-8",
            )

            self.assertEqual(
                parse_weidu_log(log),
                [Entry("MOD/SETUP-MOD.TP2", 0, 42, "Choice -> Display name: v1")],
            )

    def test_runs_split_repeated_installers_and_apply_safety_deltas(self) -> None:
        bg1 = [Entry("BG1/BG1.TP2", 0, 0, "BG1")]
        bg2 = [
            Entry("EEFIX/EEFIX.TP2", 0, 0, "fix"),
            Entry("EET/EET.TP2", 0, 0, "EET"),
            Entry("EEEX/EEEX.TP2", 0, 0, "EEex"),
            Entry("EEEX/EEEX.TP2", 0, 1, "LuaJIT in the old release"),
            Entry("MOD/MOD.TP2", 0, 1, "first"),
            Entry("EET_END/EET_END.TP2", 0, 0, "end"),
            Entry("MOD/MOD.TP2", 0, 2, "second"),
            Entry("__EXTRACT.TP2", 0, 0, "generated helper"),
        ]
        mod_ids = {
            "BG1/BG1.TP2": "bg1",
            "EEFIX/EEFIX.TP2": "eefix",
            "EET/EET.TP2": "eet",
            "EEEX/EEEX.TP2": "eeex",
            "MOD/MOD.TP2": "mod",
            "EET_END/EET_END.TP2": "eet-end",
            "BUFFBOT/SETUP-BUFFBOT.TP2": "buffbot",
        }

        runs, omissions = build_ordered_runs(bg1, bg2, mod_ids)

        self.assertEqual([run.phase for run in runs[:2]], ["bg1-preparation", "bg2-preparation"])
        self.assertEqual(
            [run.components for run in runs if run.mod_id == "mod"], [[1], [2]]
        )
        self.assertEqual(
            next(run.components for run in runs if run.mod_id == "eeex"), [0, 1, 8]
        )
        self.assertEqual(runs[-1].mod_id, "buffbot")
        self.assertEqual(runs[-1].components, [1, 0])
        self.assertEqual([entry.tp2 for entry in omissions], ["__EXTRACT.TP2"])

    def test_private_publish_roots_covers_nested_and_root_tp2s(self) -> None:
        self.assertEqual(
            private_publish_roots(
                ["A/A.TP2", "B/SETUP-B.TP2", "ROOT_FIX.TP2"]
            ),
            ["A", "B", "ROOT_FIX.TP2"],
        )

    def test_current_replacements_remove_obsolete_tails_and_removed_modpack_component(self) -> None:
        replacement_tails = [
            "FADE_FT_FIX/setup-FADE_FT_FIX.tp2",
            "FADE_FT_PATCH/setup-FADE_FT_PATCH.tp2",
            "MAZZY_PROF_FIX/MAZZY_PROF_FIX.tp2",
            "VICONIA_MULTICLASS/VICONIA_MULTICLASS.tp2",
            "XAN_EK_FIX/XAN_EK_FIX.tp2",
            "KIVAN_QUEST_FIX/setup-KIVAN_QUEST_FIX.tp2",
            "YESLICK_KELDORN_DISPEL_FIX.tp2",
            "CBM_UAI_SCROLL/CBM_UAI_SCROLL.tp2",
            "SKIE_SKILL_FIX/SKIE_SKILL_FIX.tp2",
            "NPC_KIT_CHANGES/NPC_KIT_CHANGES.tp2",
        ]
        bg2 = [
            Entry("SETUP-CHRIZ-BG-MODPACK.TP2", 0, 430, "old 430"),
            Entry("OTHER/OTHER.TP2", 0, 1, "between split modpack runs"),
            Entry("SETUP-CHRIZ-BG-MODPACK.TP2", 0, 440, "old 440"),
            Entry("SETUP-CHRIZ-BG-MODPACK.TP2", 0, 450, "old 450"),
            Entry("SETUP-CHRIZ-BG-MODPACK.TP2", 0, 600, "removed component"),
            Entry("Setup-AbettorHLARebalance.tp2", 0, 0, "retained Abettor tail"),
            Entry("AKCB_SHAPESHIFTER/setup-AKCB_SHAPESHIFTER.tp2", 0, 0, "obsolete shape tail"),
            Entry("SR_SUBSPELL_FIX/SR_SUBSPELL_FIX.tp2", 0, 0, "obsolete SR tail"),
            *(Entry(tp2, 0, 0, "replaced raw tail") for tp2 in replacement_tails),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 2001, "Minsc"),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 3101, "Ajantis"),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 5102, "Edwin"),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 7104, "Hexxat"),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 21001, "Jan"),
            Entry("ArtisansKitpack_npc/ArtisansKitpack_npc.TP2", 0, 20002, "Xan"),
            Entry("Sirene_BG2/Sirene_BG2.tp2", 0, 0, "Sirene"),
            Entry("Sirene_BG2/Sirene_BG2.tp2", 0, 2, "portrait"),
            Entry("Sirene_BG2/Sirene_BG2.tp2", 0, 6, "Cavalier"),
        ]
        mod_ids = {
            "SETUP-CHRIZ-BG-MODPACK.TP2": "chriz-bg-modpack",
            "OTHER/OTHER.TP2": "other",
            "SETUP-ABETTORHLAREBALANCE.TP2": "abettorhlarebalance",
            "ARTISANSKITPACK_NPC/ARTISANSKITPACK_NPC.TP2": "artisanskitpack-npc",
            "SIRENE_BG2/SIRENE_BG2.TP2": "sirene-bg2",
            "BUFFBOT/SETUP-BUFFBOT.TP2": "buffbot",
        }

        runs, omissions = build_ordered_runs([], bg2, mod_ids)

        self.assertEqual(
            [run.components for run in runs if run.mod_id == "chriz-bg-modpack"],
            [[component for component, _ in CURRENT_MODPACK_COMPONENTS]],
        )
        self.assertEqual(
            [run.components for run in runs if run.mod_id == "artisanskitpack-npc"],
            [[component for component, _ in CURRENT_ARTISAN_NPC_COMPONENTS]],
        )
        self.assertEqual(
            [run.components for run in runs if run.mod_id == "sirene-bg2"],
            [[component for component, _ in CURRENT_SIRENE_COMPONENTS]],
        )
        self.assertEqual(
            [run.mod_id for run in runs],
            [
                "chriz-bg-modpack",
                "other",
                "abettorhlarebalance",
                "artisanskitpack-npc",
                "sirene-bg2",
                "buffbot",
            ],
        )
        self.assertEqual(
            [(entry.tp2, entry.component) for entry in omissions],
            [
                ("SETUP-CHRIZ-BG-MODPACK.TP2", 600),
                ("AKCB_SHAPESHIFTER/setup-AKCB_SHAPESHIFTER.tp2", 0),
                ("SR_SUBSPELL_FIX/SR_SUBSPELL_FIX.tp2", 0),
                *((tp2, 0) for tp2 in replacement_tails),
                ("Sirene_BG2/Sirene_BG2.tp2", 6),
            ],
        )

    def test_abettor_root_level_tp2_includes_its_sibling_source_root(self) -> None:
        self.assertEqual(
            private_publish_roots(["Setup-AbettorHLARebalance.tp2"]),
            ["abettor-hla-rebalance", "Setup-AbettorHLARebalance.tp2"],
        )


if __name__ == "__main__":
    unittest.main()
