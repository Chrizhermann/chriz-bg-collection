# Theme fonts

The app themes ship these display and body fonts so both installers look crisp and authentic offline
without synthetic browser bolding (`font-synthesis: none`). All three families are licensed under
the SIL Open Font License 1.1; each folder holds the font's copyright line and the full licence
text in `OFL.txt`. The fonts are unmodified WOFF2 subsets taken from the Fontsource npm packages,
which repackage the Google Fonts releases.

| Folder | Font | Used by | Copyright | Source |
| --- | --- | --- | --- | --- |
| `alegreya/` | Alegreya, 400/400i/600/700, latin and latin-ext subsets | Chriz Easy BG (`../bg.css`) | Copyright 2011 The Alegreya Project Authors | `@fontsource/alegreya@5.3.0` |
| `alegreya-sans/` | Alegreya Sans, 400/500/700, latin and latin-ext subsets | Chriz Easy BG & Chriz Easy IWD (`../bg.css`, `../iwd.css`) | Copyright 2013 The Alegreya Sans Project Authors | `@fontsource/alegreya-sans@5.3.0` |
| `spectral/` | Spectral, 400/600/700, latin and latin-ext subsets | Chriz Easy IWD (`../iwd.css`) | Copyright 2017 The Spectral Project Authors | `@fontsource/spectral@5.3.0` |

SHA-256 of the shipped files:

- `alegreya/alegreya-latin-400-normal.woff2`: `7cbcf38a5ef682a8b9e2ef4a330b31a08b33bf1ae08e338267c0d1292479c68a`
- `alegreya/alegreya-latin-ext-400-normal.woff2`: `2f7d416d7c40459b8f24ee9f4d2e14b963b29c67912de517615a2f2cfe292aa4`
- `alegreya/alegreya-latin-400-italic.woff2`: `9be8976a11463895bb4d590973a684c1c96616b7e40a03333151efa8c7210241`
- `alegreya/alegreya-latin-ext-400-italic.woff2`: `27550192eeb67db7db1ff877e24b8b5e6acefe351d3b4f1941bc19c31c7029f2`
- `alegreya/alegreya-latin-600-normal.woff2`: `613096d65a0b482ba1576510fcbed26db985c67aaa76d76b233321e75d1ff1ba`
- `alegreya/alegreya-latin-ext-600-normal.woff2`: `d79944fd66b1809b2aa81da502e31a4f6f8380734dce979f200aaf376138acf0`
- `alegreya/alegreya-latin-700-normal.woff2`: `1c582501cee54f7137a80732266cbd3f0e9ef1f0e85d093228fb4f4db852df4e`
- `alegreya/alegreya-latin-ext-700-normal.woff2`: `3fe9918699e06164c1a8a28f26ce87925f769f95bb6692a852d71cf1a3906b43`
- `alegreya-sans/alegreya-sans-latin-400-normal.woff2`: `b2a5a35a2563a2f9bf9fb91939fc6ea6c115e9811cda5c4a37d02f623c7c4dbe`
- `alegreya-sans/alegreya-sans-latin-ext-400-normal.woff2`: `c76008bc2a70ed1bdfd00cf5639e6a7163b7f1a095e50aac94d8c8dcd5eab337`
- `alegreya-sans/alegreya-sans-latin-500-normal.woff2`: `55b61d22cc7afd0bca94d17a84f0e92963b8f5216dbfbde56b2f6cf530c9b2dd`
- `alegreya-sans/alegreya-sans-latin-ext-500-normal.woff2`: `578e5203186650ae0a335692f1ecc71b0591f569f3bbeae7bfcf55874e199514`
- `alegreya-sans/alegreya-sans-latin-700-normal.woff2`: `5acd19a588614d23e52b741a21dd8555178ab4932c3195c6d8fb4b296f84acae`
- `alegreya-sans/alegreya-sans-latin-ext-700-normal.woff2`: `8f289f338a955d3a9d51cd59a0a99e96627fe54d3c54859f4eeeacb9f24b80bc`
- `spectral/spectral-latin-400-normal.woff2`: `cf8daee3b83c1e662196c6e34e444bc41344d54bfeb4fb5351e197de6ce94539`
- `spectral/spectral-latin-ext-400-normal.woff2`: `fc4b2566ccd9714a72c2137f822311cb0131296c83c8ee93848ae6f887969e85`
- `spectral/spectral-latin-600-normal.woff2`: `33faca8b5795a0de1af77cbc43a050a3655c4b1e03e04847fdef4d19d2e361d2`
- `spectral/spectral-latin-ext-600-normal.woff2`: `dea6bd9ca650814af1f750b7474d7205b7fe0e4c79a6b58c6f7157d90dc8815c`
- `spectral/spectral-latin-700-normal.woff2`: `5276f202cce67e8caa9f697800e11cfc9d708da67d47f6f5507dcadcf1e11c11`
- `spectral/spectral-latin-ext-700-normal.woff2`: `9b5a5be420405c68cf73fcdd06fa4c9978dcd8fb8b87b0e187e107cfb7a5da29`

The Reserved Font Names in the OFL apply: a modified version of any font must not use its
original name. `THIRD_PARTY_NOTICES.md` carries the same attribution and licence for the
installed app.
