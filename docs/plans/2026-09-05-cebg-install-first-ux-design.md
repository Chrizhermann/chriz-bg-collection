# Chriz Easy BG install-first UX design

**Date:** 2026-09-05  
**Status:** approved by Christopher  
**Product name:** Chriz Easy BG (`CEBG`)

## Product intent

CEBG is a consumer-friendly, curated Baldur's Gate installer and launcher. It is not a
general mod manager, load-order editor, or collection builder. The primary audience includes
players who have never installed an Infinity Engine mod. The common path must therefore
make the recommended installation obvious without requiring users to discover navigation,
learn implementation vocabulary, or inspect technical diagnostics.

## Language

User-facing copy uses **install**, **installation**, **install location**, and **play**.
It does not use `campaign`, `managed copy`, `destination`, or `output`. Existing internal
engine type names may remain until a functional reason requires changing them.

The application brand is **Chriz Easy BG**. The compact header shows **CEBG 0.1 Alpha**;
the Updates/About details show the exact app and collection-recipe versions.

## Startup states

CEBG chooses its opening state from its immutable installation registry:

1. No installation: open the primary installation screen.
2. One completed, verified installation: open the launcher state for it.
3. An interrupted resumable installation: show **Continue installation** prominently.
4. Multiple completed installations: open the last-used installation and offer the small
   **Switch install** action.
5. A missing or changed registered installation: show **Installation not found** and offer
   to locate it or start a new installation; never launch an unverified path.

## Application shell

Remove the permanent left sidebar. A single quiet header contains the brand/version on the
left and **My installs** and **Updates** on the right. The main body is always the current
primary action. There is no initial navigation decision.

The default window is large enough for the primary desktop screen, while remaining within a
normal laptop display. Typography and vertical spacing are reduced from the prototype. At a
normal desktop viewport, the source summary and install action fit without a page scrollbar.
Short screens and narrow windows retain scrolling and collapse to one column.

## Primary installation screen

The opening screen contains, in order:

1. **Install Chriz Easy BG** heading and a short plain-language explanation.
2. Two compact source cards: **BG:EE + Siege of Dragonspear** and **BGII:EE**.
3. Installation name and location.
4. A short **Recommended setup selected** summary with a secondary **Customize** action.
5. A persistent readiness summary and the primary **Install Chriz Easy BG** action.

Each source card says **Found source**, displays a normal Windows path such as `C:\...`, and
shows one dominant state: **Ready** or **Needs attention**. Internal `\\?\` path prefixes are
never displayed. Detailed fingerprint and file-inventory findings live behind an accessible
**Details** disclosure. When only one candidate exists, the screen does not make the user
operate a redundant select box; multiple candidates remain selectable.

The global state is unmistakable:

- **Ready to install** when both source games are eligible, the install location is safe,
  and the recommended selection resolves. Disk space is checked again when installation starts;
  the UI does not claim an earlier space check until the native contract provides one.
- **Needs attention** with one actionable reason otherwise.

The install button is disabled until the same conditions are true. Native review/start still
repeats authoritative source, target, lock, and recipe checks; UI readiness is not authority.

## Installation identity and location

The default installation name is **Chriz Easy BG** and is editable. The default location is
`%USERPROFILE%\Games\Chriz Easy BG`. Changing the name updates the final folder name unless
the user has manually selected a full location. Changing the location preserves the chosen
display name.

The UI labels the field **Install location**, offers **Change**, shows required and available
space, and says that the original Steam/GOG installations remain unchanged. The chosen
location must be a new or empty ordinary local directory outside source games, the installer
cache, store-managed directories, and creator-protected reference paths.

The display name is included in the frozen install identity and later written to the durable
receipt/managed-install registry. Windows-invalid or empty names are rejected before review.

## Customize path

The recommended preset is the default and requires no component-by-component confirmation.
**Customize** reveals the existing reviewed feature controls. Returning from customization
preserves choices and returns to the primary installation screen. Unavailable choices and
their compatibility explanations remain visible there, not on the simple path.

## Completed installation and launcher

After verified completion, and whenever CEBG later opens with a completed installation, show:

```text
Chriz Easy BG                              My installs  Updates

Ready to play
Chriz Easy BG - Collection 0.1

[Play Chriz Easy BG]  [Open game folder]

Everything is up to date
```

**Play Chriz Easy BG** launches only the verified `InfinityLoader.exe` recorded in the
managed-install registry. **Open game folder** uses that same verified installation identity.
The exact launcher path is available under **Installation details**, not placed in the main
decision flow.

After successful completion, offer **Create desktop shortcut**, checked by default. The
shortcut opens the CEBG application/launcher rather than bypassing it, so users see update
status and the Play action. A shortcut is never created for incomplete, failed, moved, or
otherwise unverified installations.

## Visual direction

Retain the restrained dark iron, vellum, brass, and teal Baldur's Gate-inspired palette.
Spend the visual emphasis on the readiness state and one primary action. Remove the oversized
display headline, uppercase step eyebrow, decorative sidebar, repeated full-height cards, and
always-visible diagnostics. Body text uses the native Segoe family; one moderate serif heading
keeps the fantasy character without resembling a marketing landing page.

## Verification

- Test startup routing for no install, resumable install, one completed install, multiple
  installs, and stale install records.
- Test normal Windows display paths while native logic retains canonical path validation.
- Test default/editable install name and `%USERPROFILE%\Games\Chriz Easy BG` suggestion.
- Test readiness and disabled/enabled install action for every required condition.
- Test Customize preserves selection.
- Test Play, Open folder, and shortcut creation use only verified registry identities.
- Verify 1366x768 and 1920x1080 without a primary-screen scrollbar; verify scrolling and
  one-column layout at constrained sizes.
- Run frontend, native-command, engine, packaging, install/launch/uninstall smoke tests.
