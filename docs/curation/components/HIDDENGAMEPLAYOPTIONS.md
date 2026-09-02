# HIDDENGAMEPLAYOPTIONS — components

Listed at installed version **5.0** and reviewed against latest target **v5.2**.
44 target-version entries, 27 installed in the reference stack. ✓ = installed.
Subgroup = mutually exclusive; choose zero or one.

## UI/dependency notes

- The parent mod and every exposed component remain optional. Reference-installed
  components are `default`; other EET-compatible choices are `optional`.
- Component `0` is an installer convenience that bulk-installs game options `10`–`40`
  and forbids selecting them individually. Do not expose it when the individual toggles
  are available.
- Components `15`, `21`, `26`, `31`, `303`, and new-in-v5.1 component `305`
  target PST:EE or IWD:EE rather than EET and are therefore excluded. Component `31`
  additionally requires Level Up Icon Tweaks `0`.
- New-in-v5.2 component `306` supports EET only when EET GUI `0` is installed. That GUI
  is not in this collection, so keep `306` excluded. Refresh the stale v5.1 latest-version
  entry in `manifest/mod-sources.tsv` when the source pin is authored.
- Component `38` requires one Tweaks Anthology interval-save component (`3350`–`3358`).
  Component `40` requires Tweaks Anthology component `3400`.
- Component `201` is a no-log maintenance updater for component `200`, not a normal
  player preference. Keep it out of the component UI. If `200` is selected, install it
  after content mods; the orchestrator may invoke `201` internally for a later refresh.
- Every individual game-option component `10`–`40` is unavailable with EEUITweaks
  `1010`. Component `28` is also unavailable with LeUI; `29` adds Infinity UI and Classic
  BG UI exclusions; `30` additionally excludes Tipun UI. GUI components `300`–`304` and
  `306` are unavailable with LeUI variants, Dragonspear UI++, Infinity UI, Tipun UI, or
  Classic BG UI as rejected by the upstream installer.
- Components `101`–`103` are mutually exclusive. The choice group is optional, with an
  explicit **Do not update key bindings** choice; `103` is selected by default.

## Integration note

Upstream recommends installing this mod after Tweaks Anthology and GUI mods. The fresh
generated order must not replay its early reference position, especially when component
`38`, `40`, or `200` is selected.

| # | Component | Group | Subgroup | ✓ | Decision |
|---|---|---|---|---|---|
| 0 | Install all Hidden Gameplay Options at once | Game Options |  |  | |
| 10 | Add in-game option "Enable Debug Mode" | Game Options |  | ✓ | default |
| 11 | Add in-game option "Enable UI Edit Mode" | Game Options |  | ✓ | default |
| 12 | Add in-game option "Show Strrefs" | Game Options |  | ✓ | default |
| 13 | Add in-game option "Hotkeys On Tooltips" | Game Options |  | ✓ | default |
| 14 | Add in-game option "Show trigger icons on tab" | Game Options |  | ✓ | default |
| 15 | Add in-game option "Allow Spacebar in Dialogs" | Game Options |  |  | |
| 16 | Add in-game option "Limit druidic spells for Cleric/Ranger" | Game Options |  | ✓ | default |
| 17 | Add in-game option "3E Sneak Attack" | Game Options |  |  | optional |
| 18 | Add in-game option "Critical Hit Screen Shake" | Game Options |  | ✓ | default |
| 19 | Add in-game option "Show extra combat info" | Game Options |  | ✓ | default |
| 20 | Add in-game option "Show Game Date and Time on Pause" | Game Options |  | ✓ | default |
| 21 | Add in-game option "Disable Area Map Zoom" | Game Options |  |  | |
| 22 | Add in-game option "Reverse Mouse Wheel Zoom" | Game Options |  | ✓ | default |
| 23 | Add in-game option "Pause Game on Map Screen" | Game Options |  | ✓ | default |
| 24 | Add in-game option "Enable Fog" | Game Options |  | ✓ | default |
| 25 | Add in-game option "Disable Movies" | Game Options |  | ✓ | default |
| 26 | Add in-game option "No Cosmetic Attacks" | Game Options |  |  | |
| 27 | Add in-game option "XP Bonus in Nightmare Mode" | Game Options |  | ✓ | default |
| 28 | Add in-game option "Trigger Bored Sounds" | Game Options |  | ✓ | default |
| 29 | Add in-game option "Frame Rate" (experimental) | Game Options |  | ✓ | default |
| 30 | Add in-game option "Action Feedback" | Game Options |  | ✓ | default |
| 31 | Add in-game option "Display Level Up Icon" | Game Options |  |  | |
| 32 | Add in-game option "Show Area of Effect Range" | Game Options |  | ✓ | default |
| 33 | Add in-game option "Enhanced Path Search" | Game Options |  | ✓ | default |
| 34 | Add in-game option "Expire Trap Highlights" | Game Options |  | ✓ | default |
| 35 | Add in-game option "Show Learnable Spells" | Game Options |  | ✓ | default |
| 36 | Add in-game option "Render Search Map" | Game Options |  | ✓ | default |
| 37 | Add in-game option "Render Dynamic Search Map" | Game Options |  | ✓ | default |
| 38 | Add in-game options for Tweak Anthology's "Create Interval Saves" | Game Options |  |  | optional |
| 39 | Add in-game option "Force Dialog Pause" | Game Options |  | ✓ | default |
| 40 | Add in-game options for Tweaks Anthology's "Restored Loading Hints" | Game Options |  |  | optional |
| 200 | Improved Cheat Menu | Cheat Menu |  |  | optional |
| 201 | Update resource tables for "Improved Cheat Menu" | Cheat Menu |  |  | |
| 300 | Improved "Death Screen" options | GUI Improvements |  | ✓ | default |
| 301 | Improved "Quit Game" options | GUI Improvements |  | ✓ | default |
| 302 | Game Menu: Fire animation over SoD logo | GUI Improvements |  |  | optional |
| 303 | Game Menu: Improved "Quit Game" options | GUI Improvements |  |  | |
| 304 | Game Menu: Improved "Load Game" options | GUI Improvements |  |  | optional |
| 305 | Remove Show/Hide toggles from Cheat Menu (PST:EE only; added in v5.1) | GUI Improvements |  |  | |
| 306 | Remove gray bars from the log window in SoD (added in v5.2) | GUI Improvements |  |  | |
| 101 | for priest spells only | Key Bindings | Update key bindings |  | optional |
| 102 | for mage spells only | Key Bindings | Update key bindings |  | optional |
| 103 | for priest and mage spells | Key Bindings | Update key bindings | ✓ | default |
