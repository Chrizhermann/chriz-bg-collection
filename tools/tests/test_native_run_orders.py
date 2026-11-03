"""Both bundled profiles request components in each pinned TP2's declaration order.

WeiDU installs a --force-install-list batch in TP2 order whatever order it is given, and
the engine reconciles WeiDU.log against the requested order, so a run that asks for another
order stops the installation (issue 77: the "Smaller downloadable setup" profile asked for
Artisan's Kitpack 10002 before 10001 and Randomiser 1100 last). These checks read only
committed recipe files and intake evidence; they download nothing and run no installer.
"""

from pathlib import Path
import tomllib
import unittest

from tools.curated_full_recipe import BG1NPC_NATIVE_ORDER, NATIVE_RUN_ORDERS

PROFILES = ("manifest", "recipes/curated-full-current")


def _load(path: Path) -> dict:
    return tomllib.loads(path.read_text(encoding="utf-8"))


class NativeRunOrderTests(unittest.TestCase):
    root = Path(__file__).resolve().parents[2]

    def _runs(self, profile: str) -> dict[str, dict]:
        collection = _load(self.root / profile / "collection.toml")
        return {run["run_id"]: run for run in collection["runs"]}

    def _assert_native(self, run: dict, native: list[int]) -> None:
        requested = run["components"]
        self.assertFalse(set(requested) - set(native),
                         f"{run['run_id']} requests components its native order lacks")
        self.assertEqual(requested, [component for component in native if component in requested],
                         f"{run['run_id']} does not follow its TP2's declaration order")

    def test_audited_runs_follow_their_tp2_order_in_both_profiles(self):
        orders = {**NATIVE_RUN_ORDERS, "bg1npc-bg1": BG1NPC_NATIVE_ORDER}
        for profile in PROFILES:
            runs = self._runs(profile)
            for run_id, native in orders.items():
                with self.subTest(profile=profile, run=run_id):
                    self.assertIn(run_id, runs)
                    self._assert_native(runs[run_id], native)

    def test_runs_follow_the_native_order_their_intake_recorded(self):
        recorded: dict[tuple[str, str], set[tuple[int, ...]]] = {}
        for path in sorted((self.root / "manifest/evidence").glob("*.toml")):
            for item in _load(path).get("artifacts", []):
                if item.get("native_components") and item.get("artifact_id") and item.get("mod_id"):
                    key = (item["artifact_id"], item["mod_id"])
                    recorded.setdefault(key, set()).add(tuple(item["native_components"]))
        checked = 0
        for profile in PROFILES:
            runs = self._runs(profile).values()
            for mod_path in sorted((self.root / profile / "mods").glob("*.toml")):
                mod = _load(mod_path)
                orders = recorded.get((mod.get("artifact_id"), mod["id"]))
                if orders is None:
                    continue
                with self.subTest(profile=profile, mod=mod["id"]):
                    self.assertEqual(len(orders), 1, "intake records disagree on the native order")
                    native = list(next(iter(orders)))
                    for run in (run for run in runs if run["mod_id"] == mod["id"]):
                        checked += 1
                        self._assert_native(run, native)
        # Guards against the check silently matching nothing (104 runs on 2026-11-01).
        self.assertGreaterEqual(checked, 100)


if __name__ == "__main__":
    unittest.main()
