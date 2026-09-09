import { test } from "node:test";
import assert from "node:assert/strict";

import { has, integrates, lobbyEndpoint, PREVIEW_GAME, type Game } from "./games.ts";

const game = (over: Partial<Game>): Game => ({
  id: "x",
  name: "X",
  installDirs: [],
  archivePrefix: "",
  services: [],
  integrations: [],
  ...over,
} as Game);

test("a game with no lobby has nowhere to dial", () => {
  assert.equal(lobbyEndpoint(game({})), undefined);
});

test("a lobby this build cannot speak is not somewhere to dial", () => {
  /* Beyond All Reason's server is real and its address is known, but it speaks
     Tachyon. Returning it anyway would have the client open a TCP socket to a
     WebSocket endpoint and sit there until it timed out. */
  const bar = game({
    id: "bar",
    lobby: { host: "server.beyondallreason.info", port: 443, protocol: "tachyon" },
  });
  assert.equal(lobbyEndpoint(bar), undefined);
  assert.equal(has(bar, "lobby"), false, "a lobby nothing can speak was offered as a service");
});

test("a speakable lobby is handed over whole", () => {
  assert.deepEqual(lobbyEndpoint(PREVIEW_GAME), { host: "zero-k.info", port: 8200 });
});

test("an unresolved game is nowhere rather than somewhere wrong", () => {
  // The direction that matters: no answer must not become somebody else's
  // server, which is what a hardcoded default did.
  assert.equal(lobbyEndpoint(undefined), undefined);
});

test("services and integrations are absent-by-default, not present-by-default", () => {
  const bare = game({});
  assert.equal(has(bare, "profiles"), false);
  assert.equal(integrates(bare, "widgets"), false);
  assert.equal(has(undefined, "lobby"), false);
  assert.equal(integrates(undefined, "uiSkins"), false);
});
