# Add your own mods during installation

This advanced option is for a **new installation**. In **Customize → Advanced: Add
your own mods**, search the selected components and add a pause after each component
where you want to install your own additions. Multiple pauses are supported,
including between components of one mod when that boundary is available.

CEBG completes the component and closes its installer before pausing. A boundary
that cannot safely divide a mod explains why it is unavailable. If changing your
setup removes a selected boundary, remove that pause or restore the component;
CEBG will not silently move it.

Manual mods can conflict with this setup. You choose their compatibility and
install order. CEBG records your changes, but cannot guarantee them. Accepting
manual additions disables game hotpatches and whole-mod updates for this copy.
CEBG app updates and new installations remain available. A pause with no additions
does not disable updates.

## At a manual-mod pause

1. Read **Paused for your mods** and check the displayed **BG1 or BG2 folder**,
   completed component and next component. Use **Open game folder** to open that
   exact managed target. Do not install into either original source game.
2. **Back up the displayed managed game folder before making changes.** CEBG does
   not automatically create a full checkpoint backup. Keep the game closed.
   Install your additions there, one installer at a time. Keep any backups and
   diagnostics supplied by those installers. You may close
   CEBG while it is paused; the boundary is recorded on disk.
3. Close the manual installer. Choose **Check changes and continue** to inspect
   the new WeiDU components. Check their names, component numbers and target.
4. Choose **Accept additions and continue** only when the displayed additions are
   what you intended. If no additions are found, choose **Continue unchanged**.
   CEBG then resumes the remaining managed steps and retains both histories.

For example, if an add-on belongs after EEex, choose the appropriate completed EEex
component while setting up the installation. At the pause, confirm the displayed
BG2 folder, install the add-on and review its entries before continuing. This is an
ordering example, not a compatibility recommendation for any particular add-on.

BG1 pauses affect the managed BG1 copy before EET imports it. Changes made later to
BG1 do not automatically enter the combined campaign. Use a BG2 boundary for mods
that belong in the combined campaign, accounting for EET finalization and any
later content scans.

## Limits and recovery

This first version accepts **new appended WeiDU components**. Ordinary resource
and text changes made by those components are expected. Removing, reinstalling,
or reordering managed components, installing future managed components early, and
conflicting source or backup folders are rejected. File-copy-only mods are outside
this feature. Reported versions come from WeiDU's log; they are not verified archive
hashes or proof of gameplay compatibility.

The checks compare installation-log order, available diagnostics, and source/backup
names, sizes and modification times. They cannot detect every timestamp-preserving
file change. Local TP2 fingerprints identify the observed files; they do not verify
an upstream release or its provenance. Raw diagnostic notes are under **Technical
details** in the pause screen.

Never remove WeiDU.log entries to make a check pass. They describe changes still
present in the game. If checking fails, keep the folder and history intact, close
any running game/installers, and review the displayed reason. Check again after
addressing it. Keep the local installer `.DEBUG` files and the managed installation's
`.chriz/manual-checkpoints` records. A first manual pause has no final receipt yet,
so it does not offer the ordinary diagnostic ZIP export. If export is offered at
a later failure/completion state, review the ZIP before sharing. CEBG does not
automatically erase or rebuild the installation on failure.

Ordinary **Pause after current mod** saves progress for later resume. It does not
authorize manual additions. Use the planned manual-mod boundaries above.

This is a community-testing feature. Install planning, history checks and explicit
acceptance have focused automated coverage; arbitrary mod combinations and their
gameplay/UI behavior are not certified.
