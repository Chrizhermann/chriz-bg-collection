# Thief skills in armor: no-penalty QoL

## Approved behavior

Christopher wants this default-checked but optional. Armor should not lock out
ordinary thief skills or stealth; add no skill penalties or compensating bonuses.
Preserve class/kit equipment permissions, spellcasting restrictions, existing skill
values and unrelated kit abilities. CDTweaks 2100 is explicitly rejected: its PnP
penalties are not this request. Do not select it as a substitute.

## Existing candidate and ownership

Use only Klatu Tweaks 2150, **Allow Thievery in Armor**: the focused acceptance
checks below passed on 2026-09-08. The 1.7.4 source scans installed ITM resources and
removes armor-equipped opcode 144 effects for buttons 0 (stealth) and 1 (thieving),
without adding penalties or changing class/kit usability. This is source and
synthetic installer evidence, not live gameplay or full-stack acceptance.

- [Component source](https://github.com/The-Gate-Project/klatu-tweaks-and-fixes/blob/Version-1.7.4/klatu/klatu.tp2)
- [Item helper](https://github.com/The-Gate-Project/klatu-tweaks-and-fixes/blob/Version-1.7.4/klatu/lib/common.tpa)
- [Engine button definitions](https://gibberlings3.github.io/iesdp/opcodes/bgee.htm#op144)

Install after armor-adding mods and Artisan's kit patches. Artisan creates separate
Swashbuckler Insightful Strike armor restrictions; those must remain intact.
Klatu does not remove existing numerical penalties, distinct Find Traps button-14
restrictions or indirect spell/effect restrictions. Its sanitizer runs before the
armor filter and can normalize item layout beyond the armor actually unlocked.

If these limits matter for our stack, implement a narrow independent component in
**chriz-bg-modpack**, never inside this recipe repository. Do not bundle both
implementations. Do not add the rest of Klatu or enable arcane casting in armor.

## Focused acceptance — passed, no full reinstall

The unmodified released component passed real WeiDU 24900 installation against
six synthetic resources: plate, mod-added armor, Artisan-like armor, untouched
weapon/robe controls and BIF-only armor. Exact expected resource bytes confirmed
thieving/stealth lockout removal, unchanged skill modifiers, spellcasting/usability,
ability effects and Insightful Strike restrictions. The Find Traps button-14
fixture remained unchanged as expected. KEY/BIF/TLK were unchanged and uninstall
restored the original fixture byte-for-byte.

A read-only scan of the stream game's override found 170 armor items, with 63
stealth blockers, 63 thieving blockers and **zero** button-14 blockers. That scan
does not claim coverage of BIF-only items or indirect spell/effect graphs.

Ignored local evidence: `target/klatu-armor-check-20260908/result.json`,
`artifact-evidence.json`, `check.py`, and install/uninstall logs. Early fixture
setup errors (release TP2 filename and a missing HANDLE_CHARSETS game marker)
were corrected in the harness without changing the mod.

Verified release: `Version-1.7.4`, commit
`18ad2ab183e7dc71bbf481ec6331804ffd111e2f`; official IEMOD is 2,461,213 bytes,
SHA256 `7393116098ee549d9b53fdd0600d56a779ccc7feb0046667162fd10118856daa`.

Recipe integration uses **Use thief skills in armor**, explicitly **No added
skill penalties**, as one default-checked optional component, after armor/kit
changes and before the final BuffBot run. Other Klatu components stay absent.

Status: fixture accepted and integrated in draft collection alpha.14: 435 default
components across 44 runs. All 28 focused Python tests, 16 native curation/release
tests and Rust formatting passed. The native option-off test confirms disabling
Klatu restores the original run list exactly, with BuffBot still last. Public
app alpha.15 / collection alpha.13 are unchanged; no live game or save modified.
Applying this to an existing game is
a separate backed-up tail-install operation with the game closed, not an automatic
save migration or permission to copy a whole mod archive over the stream install.
