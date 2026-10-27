// Extensions here and in vite.config.ts keep this file loadable by the Vite config loader.
import type { GameRole } from "./contracts.ts";

/**
 * Product words and feature switches of each app built from this frontend. Pure data without
 * `import.meta`, so `vite.config.ts` can read it too; `flavor.ts` picks the build's flavor.
 */
export type FlavorId = "bg" | "iwd";

/** Wire slot of a source game. A standalone collection uses only `bg2` for its one game. */
export type SourceSlot = "bg1" | "bg2";

export interface SourceDefinition {
  readonly slot: SourceSlot;
  readonly role: GameRole;
  /** Title of the source card on the install screen. */
  readonly label: string;
  /** The game's name in "… source" labels and in sentences about this source. */
  readonly name: string;
  /** Term for this source on the review screen. */
  readonly reviewLabel: string;
}

export interface FlavorFeatures {
  readonly radar: boolean;
  readonly installPatches: boolean;
  /** Whole-mod upgrades such as BuffBot. */
  readonly modUpgrades: boolean;
  readonly manualPauses: boolean;
  readonly appUpdates: boolean;
  readonly buffBotTip: boolean;
  readonly radarTip: boolean;
}

/** Sentences whose wording differs beyond the product name, e.g. one game instead of two. */
export interface FlavorText {
  readonly lookingForGames: string;
  readonly preparingSetupDetail: string;
  readonly gamesIntro: string;
  readonly installIntro: string;
  readonly installSafetyNote: string;
  readonly destinationPending: string;
  readonly sourcesMissingTitle: string;
  readonly sourcesMissingDetail: string;
  readonly findingGamesAndSetup: string;
  readonly findingOriginalGames: string;
  readonly removalKeepsSources: string;
  readonly gameSettingsFailed: string;
  readonly copyGamesPhase: string;
  /** What adding manual mods means, above the pauses on the setup screen. */
  readonly manualModsWarning: string;
  /** Manual-mod guide: which folder to change at a pause. */
  readonly manualPauseFolderStep: string;
  /** Manual-mod guide: what a manual addition may change. */
  readonly manualPauseLimitsStep: string;
  /** Shown with manual additions before the player accepts them. */
  readonly manualAdditionsAcceptNote: string;
  /** Launcher note on an installation with manual mods. */
  readonly customizedLauncherNote: string;
  /** Updates-screen note on an installation with manual mods. */
  readonly customizedUpdatesNote: string;
}

export interface Flavor {
  readonly id: FlavorId;
  /** Full product name, e.g. in titles and on the Play button. */
  readonly productName: string;
  /** The name used in running sentences. */
  readonly shortName: string;
  /** The game or games this app installs. */
  readonly gameName: string;
  readonly sources: readonly SourceDefinition[];
  readonly features: FlavorFeatures;
  readonly text: FlavorText;
}

export const FLAVORS: Readonly<Record<FlavorId, Flavor>> = {
  bg: {
    id: "bg",
    productName: "Chriz Easy BG",
    shortName: "CEBG",
    gameName: "Baldur's Gate",
    sources: [
      { slot: "bg1", role: "bgee_sod", label: "Baldur's Gate + Siege of Dragonspear", name: "Baldur's Gate", reviewLabel: "BG1 source" },
      { slot: "bg2", role: "bg2ee", label: "Baldur's Gate II", name: "Baldur's Gate II", reviewLabel: "BG2 source" },
    ],
    features: {
      radar: true,
      installPatches: true,
      modUpgrades: true,
      manualPauses: true,
      appUpdates: true,
      buffBotTip: true,
      radarTip: true,
    },
    text: {
      lookingForGames: "Looking for your Baldur's Gate games…",
      preparingSetupDetail: "Checking your game folders and mod choices. Your original games stay unchanged.",
      gamesIntro: "Choose your original Baldur's Gate games. CEBG will check that they are ready to use.",
      installIntro: "Your games, with Chriz's recommended mods. Ready to install in a separate folder.",
      installSafetyNote: "Your original games stay unchanged. CEBG installs into this separate folder.",
      destinationPending: "The installer will verify that it is separate from both clean source games.",
      sourcesMissingTitle: "Choose both game sources",
      sourcesMissingDetail: "Use Change source for each missing game before CEBG checks this install location.",
      findingGamesAndSetup: "CEBG is finding your games and recommended setup.",
      findingOriginalGames: "CEBG is finding your original games.",
      removalKeepsSources: "Your original game sources and shared downloads stay untouched. Saves stored outside this folder are kept.",
      gameSettingsFailed: "Lefreut's interfaces need this update to offer the trilogy campaigns. Close the game, then use Play again to retry.",
      copyGamesPhase: "Copy your games",
      manualModsWarning: "Manual mods can conflict with this setup. You choose their compatibility and install order. CEBG records your changes, but cannot guarantee them. Adding manual mods disables game hotpatches and whole-mod updates for this installation; CEBG app updates still work.",
      manualPauseFolderStep: "At the pause, use the displayed managed BG1 or BG2 folder, never either original game. Back up that folder before making changes: CEBG does not automatically create a full checkpoint backup. Keep the game closed and retain the manual installer’s backups and diagnostics.",
      manualPauseLimitsStep: "Only new WeiDU components are supported. Do not remove, reinstall or reorder managed components or edit WeiDU.log. BG1 changes after EET imports it will not enter the combined campaign.",
      manualAdditionsAcceptNote: "Accepting records these additions and disables game hotpatches and whole-mod updates for this installation. Compatibility is your responsibility. App updates still work.",
      customizedLauncherNote: "Customized — manual mods added. Game hotpatches and whole-mod updates are disabled; CEBG app updates still work.",
      customizedUpdatesNote: "Customized — manual mods added. Game hotpatches and whole-mod updates are disabled for this installation. CEBG app updates and new installations remain available.",
    },
  },
  iwd: {
    id: "iwd",
    productName: "Chriz Easy IWD",
    shortName: "Chriz Easy IWD",
    gameName: "Icewind Dale: Enhanced Edition",
    sources: [
      { slot: "bg2", role: "iwdee", label: "Icewind Dale: Enhanced Edition", name: "Icewind Dale: Enhanced Edition", reviewLabel: "Game source" },
    ],
    features: {
      radar: true,
      installPatches: false,
      modUpgrades: false,
      manualPauses: true,
      appUpdates: false,
      buffBotTip: true,
      radarTip: true,
    },
    text: {
      lookingForGames: "Looking for your Icewind Dale game…",
      preparingSetupDetail: "Checking your game folder and mod choices. Your original game stays unchanged.",
      gamesIntro: "Choose your original Icewind Dale: Enhanced Edition game. Chriz Easy IWD will check that it is ready to use.",
      installIntro: "Your game, with Chriz's recommended mods. Ready to install in a separate folder.",
      installSafetyNote: "Your original game stays unchanged. Chriz Easy IWD installs a separate copy into this folder, with its own game profile.",
      destinationPending: "The installer will verify that it is separate from your clean source game.",
      sourcesMissingTitle: "Choose your game source",
      sourcesMissingDetail: "Use Change source to choose your game before Chriz Easy IWD checks this install location.",
      findingGamesAndSetup: "Chriz Easy IWD is finding your game and recommended setup.",
      findingOriginalGames: "Chriz Easy IWD is finding your original game.",
      removalKeepsSources: "Your original game and shared downloads stay untouched. Saves stored outside this folder are kept.",
      gameSettingsFailed: "Close the game, then use Play again to retry.",
      copyGamesPhase: "Copy your game",
      // Manual mods turn off the installed-mod check, the only update of a game in this app.
      manualModsWarning: "Manual mods can conflict with this setup. You choose their compatibility and install order. Chriz Easy IWD records your changes, but cannot guarantee them. Adding manual mods turns off mod update checks for this installation.",
      manualPauseFolderStep: "At the pause, use the displayed managed game folder, never your original game. Back up that folder before making changes: Chriz Easy IWD does not automatically create a full checkpoint backup. Keep the game closed and retain the manual installer’s backups and diagnostics.",
      manualPauseLimitsStep: "Only new WeiDU components are supported. Do not remove, reinstall or reorder managed components or edit WeiDU.log.",
      manualAdditionsAcceptNote: "Accepting records these additions and turns off mod update checks for this installation. Compatibility is your responsibility.",
      customizedLauncherNote: "Customized — manual mods added. Mod update checks are off for this installation.",
      customizedUpdatesNote: "Customized — manual mods added. Mod update checks are off for this installation. New installations remain available.",
    },
  },
};

/** The page's theme stylesheet for each app; `index.html` links the BG one. */
export function themeStylesheet(id: FlavorId): string {
  return `/src/themes/${id}.css`;
}

/** The product words and theme that `index.html` carries for the BG app, and their replacements. */
export function indexHtmlReplacements(target: FlavorId): readonly (readonly [string, string])[] {
  const from = FLAVORS.bg;
  const to = FLAVORS[target];
  return target === "bg" ? [] : [[from.productName, to.productName], [themeStylesheet("bg"), themeStylesheet(target)]];
}
