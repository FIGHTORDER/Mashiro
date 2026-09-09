/**
 * Which game Mashiro is presenting itself as.
 *
 * The registry lives in Rust (`src-tauri/src/games.rs`) and this is the typed
 * view of it. Deliberately not a second copy of the table: a list of games in
 * both languages drifts, and the half that drifts is always the one nobody
 * looked at. Rust resolves the active game from the install it actually found,
 * so what the screens offer matches the files on disk.
 */
import { invoke } from "@tauri-apps/api/core";

import { inTauri } from "./connection.ts";

/** A network service a game may or may not have. */
export type Service =
  | "lobby"
  | "contentSearch"
  | "battleHistory"
  | "profiles"
  | "codex"
  | "campaign";

/** A way of reaching into the game's own in-game interface. */
export type Integration = "widgets" | "uiSkins" | "loadScreen" | "lobbyButton";

export interface Game {
  id: string;
  name: string;
  installDirs: string[];
  steamAppId?: number;
  rapidTag?: string;
  archivePrefix: string;
  /** The game's own site, no trailing slash. Reaches copy and outbound links. */
  site?: string;
  /**
   * Where the lobby is and what it speaks.
   *
   * Present even when `services` omits `lobby`: Beyond All Reason has a real
   * server this build cannot talk to, and describing it honestly is different
   * from pretending it has none. Only `zkLobby` is speakable here.
   */
  lobby?: { host: string; port: number; protocol: "zkLobby" | "tachyon" };
  services: Service[];
  integrations: Integration[];
}

/**
 * What to assume before Rust has answered, and outside Tauri.
 *
 * No services at all, so a screen that needs one stays hidden rather than
 * appearing for a moment and then disappearing. The first paint being a little
 * bare is a much smaller problem than the nav rail changing shape under a
 * cursor that is already moving toward an item.
 */
export const UNKNOWN_GAME: Game = {
  id: "unknown",
  name: "Recoil",
  installDirs: [],
  archivePrefix: "",
  services: [],
  integrations: [],
};

/**
 * What the client looks like with no machine to inspect.
 *
 * Outside Tauri there is no install, no registry and no way to ask - the
 * browser preview and the e2e suite both run here. Answering `UNKNOWN_GAME`
 * there would hide two thirds of the client, which makes the preview useless
 * for looking at the thing being built and is not a truthful answer either:
 * "nothing is installed" is not the same as "this client cannot do that".
 *
 * So the preview claims every service, and every integration for the same
 * reason - the Add-ons kinds are gated on those, and a demo that hid Widgets
 * would be hiding a panel the client has. Nothing behind them is real: the demo
 * fixtures stand in, and nothing here is used inside Tauri, where the registry
 * gives a real answer from a real install.
 */
export const PREVIEW_GAME: Game = {
  id: "preview",
  name: "Recoil",
  installDirs: [],
  archivePrefix: "",
  lobby: { host: "zero-k.info", port: 8200, protocol: "zkLobby" },
  services: ["lobby", "contentSearch", "battleHistory", "profiles", "codex", "campaign"],
  integrations: ["widgets", "uiSkins", "loadScreen", "lobbyButton"],
};

/**
 * The lobby to dial for this game, if this build can speak to it.
 *
 * `undefined` for a game with no lobby, and for one whose protocol nothing here
 * implements - a caller with nowhere to connect should say so rather than fall
 * back to somebody else's server, which is what a hardcoded default did.
 */
export function lobbyEndpoint(game: Game | undefined): { host: string; port: number } | undefined {
  const lobby = game?.lobby;
  if (!lobby || lobby.protocol !== "zkLobby") return undefined;
  return { host: lobby.host, port: lobby.port };
}

export function has(game: Game | undefined, service: Service): boolean {
  return game?.services?.includes(service) ?? false;
}

export function integrates(game: Game | undefined, i: Integration): boolean {
  return game?.integrations?.includes(i) ?? false;
}

/** Every game this build knows about. */
export function games(): Promise<Game[]> {
  if (!inTauri()) return Promise.resolve([]);
  return invoke<Game[]>("zks_games");
}

/**
 * The game to present.
 *
 * Never rejects: a machine with no install at all still has a client to look
 * at, and the honest answer there is "nothing is known", not an error banner.
 */
export function activeGame(installRoot?: string): Promise<Game> {
  if (!inTauri()) return Promise.resolve(PREVIEW_GAME);
  return invoke<Game>("zks_active_game", { installRoot }).catch(() => UNKNOWN_GAME);
}
