# BG Radar Overlay integration evidence

Verified 2026-09-05 for the Chriz Easy BG add-on integration. This is a separate launcher
add-on, not a WeiDU mod and not part of `manifest/install-order.tsv`.

## Authoritative upstream

- Repository: <https://github.com/tapahob/BG2RadarOverlay>
- License: MIT (`LICENSE.md`, copyright 2022 Aleksandra Lukianchenko)
- GitHub Latest release: id `381826828`, tag `2.1.0.0`, published
  `2026-09-03T07:41:37Z`, normal non-draft/non-prerelease release
- Release page: <https://github.com/tapahob/BG2RadarOverlay/releases/tag/2.1.0.0>
- Release commit shown by GitHub: `0c797f3`
- Only release asset: id `543014181`, `BG.Radar.Overlay.7z.7z`, 68,133,036 bytes
- GitHub-advertised and independently verified SHA-256:
  `8cd0348011638398d14d005fdda620032ca5ffa8833c86586d209229e3e567ce`

The asset is a real 7z archive despite the doubled `.7z.7z` name. Recent upstream
releases also use 7z rather than ZIP, so the integration must not treat the download as
ZIP-compatible.

## Upstream behavior and payload

The upstream README says to extract the archive and start `BG Radar Overlay.exe`; the
folder no longer matters because the overlay detects the running game. It lists BG1EE and
BG2EE 2.6/2.7 support, requires ModMerge/DLCMerge for SoD, and also lists IWDEE 2.7.

The verified `2.1.0.0` archive has six files at its root:

| Path | Bytes | SHA-256 |
|---|---:|---|
| `BG Radar Overlay.exe` | 74,787,668 | `a58cd875e5b335301974a244b3bc96016e6bb6ea973d734a23277c6643402360` |
| `Locales/en_us.txt` | 2,136 | `b3762763f8b958b91aeb8afd8721029119204a42f92b48d0e5a04bf3eafe945a` |
| `Locales/pl_PL.txt` | 1,999 | `96caec0c0c005bcae685c3b533b7daa7faf7a39e6f540c84c8e1d9f7fcb60832` |
| `Locales/ru_ru.txt` | 2,648 | `16c4b0870c465d7d4f609db32037450fd3b81cb21d940c2455f5d87bb3027130` |
| `Locales/tr_TR.txt` | 1,992 | `15e87c528d3a0b3ed7fb7ed1a877538403774d2e9c29bbfd0d3d8eafeb1e51e7` |
| `Locales/zh_CN.txt` | 1,759 | `337f34e7d2755d68a2f71fb0bba367907360668a2dd9757f2ad07556c5dcd93a` |

The executable's Windows FileVersion and ProductVersion are both `2.1.0.0`.

## CEBG ownership and verification boundary

CEBG installs the payload under `<managed>/game/BG Radar Overlay/` and records exact
owned file lengths and hashes at `<managed>/.chriz/addons/bg-radar-overlay.json`.
An update is allowed only when every previously owned file is unchanged. Unowned runtime
files such as `config.cfg` are preserved; an unowned collision with a new release file is
rejected. The archive is downloaded through the existing content-addressed cache using the
GitHub length and digest, then path/entry/expanded-size bounds are checked before 7z
extraction and transactional directory replacement.

Static acceptance used the exact GitHub asset in a disposable managed root. The engine's
native downloader cache, 7z parser, extraction inventory, publication receipt, and status
check completed successfully; the installed six-file payload included the expected
74,787,668-byte executable. GitHub Latest metadata was also fetched live through the final
engine API and matched the ids, tag, size, and digest above. The overlay executable was not
launched and no in-game behavior was tested. No path under `C:\Games` was read or written
for this work.
