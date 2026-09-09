/**
 * The Spring rapid index, as games somebody can install.
 *
 * Rust fetches and filters it - see `src-tauri/src/rapidrepos.rs` for what is
 * in the index and why libraries are excluded by name rather than by tag.
 *
 * Installing one goes through the ordinary content path: a rapid tag is exactly
 * what `pr-downloader` takes, so there is no separate downloader here.
 */
import { invoke } from "@tauri-apps/api/core";

import { inTauri } from "./connection.ts";

export interface RapidGame {
  /** Repository id, e.g. `s44`. */
  repo: string;
  /** What to call it. The repository id when nothing better is known. */
  name: string;
  /** The rapid tag to download, e.g. `s44:stable`. */
  tag: string;
}

/**
 * A few real entries, for the browser preview.
 *
 * The demo has no Rust behind it, and an empty list would make the screen look
 * broken rather than unpopulated. Taken from the live index so the shape and
 * the names are the real ones.
 */
const PREVIEW: RapidGame[] = [
  { repo: "ba", name: "Balanced Annihilation", tag: "ba:stable" },
  { repo: "evo", name: "Evolution RTS", tag: "evo:stable" },
  { repo: "metalfactions", name: "Metal Factions", tag: "metalfactions:stable" },
  { repo: "s44", name: "Spring: 1944", tag: "s44:stable" },
  { repo: "techa", name: "Tech Annihilation", tag: "techa:stable" },
  { repo: "xta", name: "XTA", tag: "xta:stable" },
  { repo: "zk", name: "Zero-K", tag: "zk:stable" },
];

/**
 * Every game the index offers.
 *
 * Rejects rather than returning nothing when the index cannot be read: an empty
 * catalogue and an unreachable one look identical on screen, and only one of
 * them is worth a retry button.
 */
export function rapidGames(): Promise<RapidGame[]> {
  if (!inTauri()) return Promise.resolve(PREVIEW);
  return invoke<RapidGame[]>("zks_rapid_games");
}

/**
 * The game archives already on this machine, by their own names.
 *
 * Empty outside Tauri and empty on a machine with no install: both mean "we
 * cannot say", and an empty list marks nothing rather than marking everything.
 */
export function installedGames(installRoot?: string): Promise<string[]> {
  if (!inTauri()) return Promise.resolve([]);
  return invoke<string[]>("zks_installed_games", { installRoot }).catch(() => []);
}

/**
 * Is this catalogue entry one of the archives already here?
 *
 * The two sides do not spell the same thing. A catalogue entry is
 * `Balanced Annihilation`; the archive is `Balanced Annihilation V15.9.8`,
 * because the engine indexes versions. So this folds both to letters and digits
 * and asks whether the archive starts with the catalogue name.
 *
 * Not a substring test: `XTA` appears inside `Tech Annihilation + Flea Spam`,
 * and a substring would mark XTA installed for somebody who never had it.
 *
 * Not a bare prefix either, which is subtler and was wrong here first. The
 * catalogue really does carry both `Tech Annihilation` and `Tech Annihilation +
 * Flea Spam` - repositories `techa` and `tafs` - so a prefix test marked the
 * first installed for anybody who had only the second. What follows the name
 * has to look like a *version*: nothing at all, or digits, optionally behind a
 * `v`. `V15.9.8` qualifies; `+ Flea Spam v3.8` does not.
 */
export function installedAlready(game: RapidGame, archives: string[]): boolean {
  const fold = (t: string) => t.toLowerCase().replace(/[^a-z0-9]/g, "");
  const want = fold(game.name);
  if (!want) return false;
  return archives.some(a => {
    const folded = fold(a);
    if (!folded.startsWith(want)) return false;
    const rest = folded.slice(want.length);
    // Exactly this game, or this game and a version.
    return rest === "" || /^v?[0-9]/.test(rest);
  });
}
