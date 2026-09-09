import { test } from "node:test";
import assert from "node:assert/strict";

import { KINDS, visibleKinds } from "./addonKinds.ts";
import type { Game } from "../net/games.ts";

/** A game carrying exactly the integrations named, and nothing else. */
const game = (id: string, integrations: string[]): Game => ({
  id,
  name: id,
  installDirs: [id],
  archivePrefix: id,
  services: [],
  integrations,
} as unknown as Game);

const ZK = game("zk", ["widgets", "uiSkins", "loadScreen", "lobbyButton"]);
const BARE = game("bar", []);

test("a game with no integrations keeps the kinds that are ours, not its", () => {
  /* `games` is the rapid index, which belongs to the ecosystem rather than to
     any one game - a player with any Recoil install can fetch any Spring game
     from it, so it is never gated. */
  const ids = visibleKinds(BARE).map(k => k.id);
  assert.deepEqual(ids, ["games", "apps", "skins", "campaign"]);
});

test("Zero-K sees every kind", () => {
  assert.equal(visibleKinds(ZK).length, KINDS.length);
});

test("nothing that writes into the game's own interface survives without it", () => {
  /* The point of the gate. `ZK_data.lua` and Chili are Zero-K's architecture;
     offered for another game they would write files nothing there reads, and
     report success doing it. */
  const ids = visibleKinds(BARE).map(k => k.id);
  for (const id of ["widgets", "uiskins", "loadscreens"]) {
    assert.ok(!ids.includes(id), `${id} was offered to a game that has no such interface`);
  }
});

test("the kind being looked at is never taken away underfoot", () => {
  const ids = visibleKinds(BARE, "widgets").map(k => k.id);
  assert.ok(ids.includes("widgets"));
});

test("an unknown game is treated as having nothing rather than everything", () => {
  // The safe direction: showing a screen with nothing behind it is worse than
  // hiding one that might have worked.
  const ids = visibleKinds(undefined).map(k => k.id);
  assert.deepEqual(ids, ["games", "apps", "skins", "campaign"]);
});
