# Creator-full parity

- BG1 source entries: 28.
- BG2 source entries: 456.
- Generated executable runs: 90.
- Manual/private installer payloads: 49 in `creator-full-private-extras-20260902.zip`.
- Intentional source-log deltas: omit the unavailable Project Infinity pseudo-installers `__EXTRACT.TP2` and `__IDS.TP2`; replace the reference modpack selection with the approved current components 110, 130, 140, 170, 190, 192-198, 410, 430, 440, and 450; omit removed modpack component 600 because v0.2.0-alpha.1 has no equivalent; replace the raw Fade, Mazzy, Viconia, Xan, Kivan, Yeslick/Keldorn, UAI-scroll, and Skie tails with maintained modpack components; replace the unsafe numeric-kit NPC monolith with semantic Artisan/modpack assignments and Sirene's approved True Paladin choice; omit the obsolete Shapeshifter and SR standalone tail installers; add EEex 1.2 component 8 (LuaJIT); append BuffBot components 1 then 0 as the absolute final run.
- Direct maintained mappings: both Fade tails -> modpack 110; Kivan -> 130; Mazzy -> 140; Xan -> 170; Viconia -> 192; Skie -> 195; Yeslick/Keldorn -> 410; and the UAI-scroll tail -> 430. Modpack 195 includes both Skie's semantic Swashbuckler assignment and the stealth-to-lock-picking transfer, so `SKIE_SKILL_FIX` is not replayed.
- The unsafe `NPC_KIT_CHANGES` monolith is not replayed beside semantic components. Khalid, Shar-Teel, Kagain, Sarah, Skie, Imoen, and Mazzy map to current Artisan/modpack components; Sirene deliberately uses the approved True Paladin component 5 instead of the retired Martyr rewrite. Its Safana-to-Abettor and Aura-to-Bard assignments remain unmapped outcome differences.
- Replacement evidence: Artisan chriz-v1.3.1 already sets every greater-werewolf template to `personal_space=3`; Spell Revisions v4.21-chriz.3 migrates hidden elemental subspells itself. The standalone Abettor semantic rebalance is retained with its sibling source root. The old component-600 Cat and Mouse behavior and the monolith's Safana-to-Abettor and Aura-to-Bard assignments have no approved current mappings and remain explicit outcome differences.
- All overlapping public-alpha artifacts use the newer pinned artifact manifests.
- This recipe is private/manual authoring data. The repository and any public recipe package contain no third-party mod payloads.
