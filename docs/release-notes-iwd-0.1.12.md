# Chriz Easy IWD 0.1.12

**More flexible path, drive, OS metadata, archive picker, and hotpatch handling — plus
smoother path typing and clearer antivirus guidance.** App **0.1.12** (bundled collection
**0.1.10**, 30 mods with 140 components and 18 companions) brings 17 reliability and
flexibility improvements from our Windows path and setup audit. Updating the app leaves
installed games and saves unchanged.

## Highlights

- **More flexible Windows folders, drives & junctions.** Accepts parent folder junctions,
  OneDrive/cloud folders, mapped drives and trailing-slash/case spelling differences while
  still blocking unsafe direct links, and replaces existing diagnostic ZIPs safely on exFAT
  and FAT32 drives (`os error 1`).
- **Ignores OS metadata in empty destination folders.** Destinations containing only
  `desktop.ini`, `Thumbs.db`, `.DS_Store`, `.__*`, or `Icon\r` count as empty without
  requiring an extra inner subfolder.
- **Broader archive picker & smoother input typing.** Manual mod selection accepts `.zip`,
  `.iemod`, `.exe`, `.rar`, and `.7z` files consistently, and typing in install name or
  folder path fields preserves focus and cursor selection across live re-validation.
- **Tolerant hotpatch checks & Windows Security guidance.** Hotpatch checks tolerate normal
  game runtime files and temporary save writes, and missing `InfinityLoader.exe` on launch or
  load surfaces clear Windows Security quarantine restoration guidance.

On app **0.1.7** to **0.1.11**, use **Updates → Update Chriz Easy IWD app**. On
**0.1.1–0.1.6**, install `Chriz.Easy.IWD_0.1.12_x64-setup.exe` by hand once.

Windows 10 or 11 (64-bit), Icewind Dale: Enhanced Edition 2.7.3.0 and about 8 GB of free
space are required. Keep the computer awake during installation.
