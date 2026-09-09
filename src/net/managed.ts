/**
 * Zero-K installed by Shiro, rather than found.
 *
 * The other half of `install` detection: when there is no Zero-K on the
 * machine, Shiro can make one. The engine comes from Zero-K's own server as a
 * 45 MB zip, and `pr-downloader` is inside it - so one download turns an empty
 * folder into something that can fetch the game and every map after it. The
 * rest of the pipeline is the one that already exists.
 *
 * Nothing here starts on its own. Setting up an install means gigabytes, so it
 * happens because somebody pressed a button in Settings.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

import { inTauri } from "./connection.ts";

export interface ManagedState {
  root: string;
  /** The directory exists and Shiro made it. */
  prepared: boolean;
  /** An engine of the version asked about is installed there. */
  engineInstalled: boolean;
  /** Archives the engine has scanned there. Zero on a fresh one. */
  archives: number;
}

export type EngineStatus =
  | { kind: "started"; version: string }
  | { kind: "progress"; received: number; total: number }
  | { kind: "done"; version: string; path: string }
  | { kind: "failed"; reason: string };

/** Where a managed install would go, whether or not one is there. */
export async function managedRoot(gameId?: string): Promise<string> {
  if (!inTauri()) return "";
  /* One directory per game - see `managed::root_for`. Omitted means the
     default, which is Zero-K and is where every install made before this went. */
  return invoke<string>("zks_managed_root", { gameId });
}

export async function managedState(
  engineVersion?: string, gameId?: string,
): Promise<ManagedState | null> {
  if (!inTauri()) return null;
  return invoke<ManagedState>("zks_managed_state", { engineVersion, gameId });
}

/** Create the directory, before anything large starts arriving in it. */
export async function prepareManaged(gameId?: string): Promise<string> {
  if (!inTauri()) return "";
  return invoke<string>("zks_managed_prepare", { gameId });
}

/**
 * Fetch and unpack the engine the server asked for.
 *
 * The version is never chosen here - it is the one in `Welcome.Engine`, so the
 * only engine ever downloaded is the one a game is about to need.
 */
export async function installEngine(version: string, gameId?: string): Promise<string> {
  if (!inTauri()) return "";
  return invoke<string>("zks_managed_install_engine", { version, gameId });
}

/** Delete a managed install. Refused unless Mashiro made the directory. */
export async function removeManaged(gameId?: string): Promise<void> {
  if (!inTauri()) return;
  await invoke("zks_managed_remove", { gameId });
}

export function onEngine(cb: (s: EngineStatus) => void): Promise<UnlistenFn> {
  return listen<EngineStatus>("zks://engine", e => cb(e.payload));
}

/**
 * Shiro's loading screen: the file Zero-K's own LuaIntro finds before its own.
 *
 * Only offered for an install Shiro made. Somebody else's Zero-K directory is
 * theirs, and a launcher writing files into it uninvited is not something to do
 * quietly.
 */
export async function loadScreenState(): Promise<boolean> {
  if (!inTauri()) return false;
  return invoke<boolean>("zks_loadscreen_state");
}

export async function setLoadScreen(enabled: boolean): Promise<boolean> {
  if (!inTauri()) return false;
  return invoke<boolean>("zks_loadscreen_set", { enabled });
}
