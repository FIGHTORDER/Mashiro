import { strict as assert } from "node:assert";
import { test } from "node:test";

import { NAV, visibleNav } from "./navItems.ts";
import { PREVIEW_GAME, UNKNOWN_GAME, type Game } from "../net/games.ts";

const zk: Game = {
  id: "zk",
  name: "Zero-K",
  installDirs: ["Zero-K"],
  archivePrefix: "Zero-K",
  services: ["lobby", "contentSearch", "battleHistory", "profiles", "codex", "campaign"],
  integrations: ["widgets", "uiSkins", "loadScreen", "lobbyButton"],
};

const bar: Game = {
  id: "bar",
  name: "Beyond All Reason",
  installDirs: ["Beyond All Reason"],
  archivePrefix: "Beyond All Reason",
  services: [],
  integrations: [],
};

const ids = (...args: Parameters<typeof visibleNav>) => visibleNav(...args).map(n => n.id);

test("a game with every service gets the whole rail", () => {
  const shown = ids({ game: zk, hasCampaigns: true, hasGalaxy: true });
  for (const item of NAV) {
    assert.ok(shown.includes(item.id), `${item.id} is missing for Zero-K`);
  }
});

test("a game with no services keeps only what the engine provides", () => {
  /* The point of the whole registry. Maps, replays, the last match and add-ons
     read the data directory and the .sdfz format, which every Recoil game
     shares - so they stay. Battles, chat, matchmaker, friends, profile and the
     codex all need a service Beyond All Reason has no implementation for here,
     and a screen with nothing behind it is worse than no screen. */
  const shown = ids({ game: bar, hasCampaigns: false, hasGalaxy: false });
  assert.deepEqual(shown, ["maps", "debrief", "replays", "apps"]);
});

test("an unknown game shows nothing that needs a service", () => {
  // What the first paint looks like, before Rust has answered.
  const shown = ids({ game: UNKNOWN_GAME });
  assert.deepEqual(shown, ["maps", "debrief", "replays", "apps"]);
  assert.ok(!shown.includes("battles"), "a lobby screen with no lobby");
});

test("no game at all is the same as an unknown one, not a crash", () => {
  assert.deepEqual(ids({}), ["maps", "debrief", "replays", "apps"]);
});

test("the screen being looked at is never taken away underfoot", () => {
  /* Switching to a game with no codex while standing on the Codex screen must
     not remove the item under the cursor - App.jsx moves the view first, and
     until it does the item stays. */
  const shown = ids({ game: bar, view: "codex" });
  assert.ok(shown.includes("codex"));
});

test("the galaxy needs both the service and something readable", () => {
  /* Two different questions: does this game have a campaign at all, and could
     this machine actually read it. Zero-K with no install answers yes then no. */
  assert.ok(!ids({ game: zk, hasGalaxy: false }).includes("galaxy"));
  assert.ok(ids({ game: zk, hasGalaxy: true }).includes("galaxy"));
  assert.ok(!ids({ game: bar, hasGalaxy: true }).includes("galaxy"),
    "BAR has no campaign service, so a readable one is not enough");
});

test("every gated item names a service some registered game has", () => {
  /* Guards against a typo in `needs`, which would hide a screen for every game
     and look exactly like a deliberate gate. */
  const offered = new Set([...zk.services, ...bar.services]);
  for (const item of NAV) {
    if (item.needs) {
      assert.ok(offered.has(item.needs), `${item.id} needs "${item.needs}", which no game has`);
    }
  }
});

test("the browser preview shows the whole client, unlike an unknown game", () => {
  /* Two different "we do not know" answers, and they must not be the same one.
     Inside Tauri, before the registry replies, hiding a screen is right - it
     stops the rail changing shape under a moving cursor. Outside Tauri there is
     no install to inspect at all, and hiding two thirds of the client would
     make the browser preview useless for looking at what is being built. */
  const preview = visibleNav({ game: PREVIEW_GAME }).map(n => n.id);
  const unknown = visibleNav({ game: UNKNOWN_GAME }).map(n => n.id);

  assert.ok(preview.includes("battles"), "the preview hides the lobby");
  assert.ok(preview.includes("codex"));
  assert.ok(!unknown.includes("battles"), "an unknown game claims a lobby");
  assert.ok(preview.length > unknown.length);
});
