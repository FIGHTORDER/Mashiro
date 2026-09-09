import { test } from "node:test";
import assert from "node:assert/strict";

import { formatServer, parseServer, resolveServer } from "./serverAddress.ts";

const ZK_PORT = 8200;

test("a bare host takes the default port", () => {
  assert.deepEqual(parseServer("lobby.example.org", ZK_PORT).server,
    { host: "lobby.example.org", port: 8200 });
});

test("a port after a colon is used", () => {
  assert.deepEqual(parseServer("lobby.example.org:9000", ZK_PORT).server,
    { host: "lobby.example.org", port: 9000 });
});

test("whitespace around it is not part of the address", () => {
  assert.deepEqual(parseServer("  lobby.example.org:9000  ", ZK_PORT).server,
    { host: "lobby.example.org", port: 9000 });
});

test("empty means the default, not an error", () => {
  /* The field starts filled from the game and somebody may clear it. That is
     "use whatever the game says", not something to complain about. */
  const got = parseServer("   ", ZK_PORT);
  assert.equal(got.server, undefined);
  assert.equal(got.error, undefined);
});

test("a bad port is refused rather than quietly replaced", () => {
  /* The one outcome worse than refusing is dialling a different machine than
     the one somebody typed. */
  for (const bad of ["lobby.example.org:0", "lobby.example.org:70000",
    "lobby.example.org:eight", "lobby.example.org:82.5"]) {
    const got = parseServer(bad, ZK_PORT);
    assert.equal(got.server, undefined, bad);
    assert.match(got.error || "", /port/, bad);
  }
});

test("a pasted URL is refused rather than half-understood", () => {
  /* This is a raw TCP lobby. Dropping the scheme would dial port 8200 on a host
     somebody believed was reached over TLS. */
  for (const url of ["https://lobby.example.org", "wss://lobby.example.org:443",
    "http://lobby.example.org:8200"]) {
    const got = parseServer(url, ZK_PORT);
    assert.equal(got.server, undefined, url);
    assert.match(got.error || "", /host/, url);
  }
});

test("an IPv6 address works when it is bracketed", () => {
  assert.deepEqual(parseServer("[::1]:8200", ZK_PORT).server, { host: "::1", port: 8200 });
  assert.deepEqual(parseServer("[::1]", ZK_PORT).server, { host: "::1", port: 8200 });
});

test("a bare IPv6 address is explained rather than mangled", () => {
  // Splitting on the first colon would give host "" and port ":1".
  const got = parseServer("::1", ZK_PORT);
  assert.equal(got.server, undefined);
  assert.match(got.error || "", /brackets/);
});

test("a host with a space in it is not a host", () => {
  assert.match(parseServer("my server", ZK_PORT).error || "", /spaces/);
});

test("the field hides a port that is the game's own", () => {
  // Less to read, and nothing lost: clearing it gives the same server back.
  assert.equal(formatServer({ host: "zero-k.info", port: 8200 }, ZK_PORT), "zero-k.info");
  assert.equal(formatServer({ host: "zero-k.info", port: 9000 }, ZK_PORT), "zero-k.info:9000");
  assert.equal(formatServer(undefined, ZK_PORT), "");
});

test("what was typed wins over the game's own lobby", () => {
  const typed = { host: "test.example.org", port: 9000 };
  const game = { host: "zero-k.info", port: 8200 };
  assert.deepEqual(resolveServer(typed, game), typed);
  assert.deepEqual(resolveServer(undefined, game), game);
});

test("a game with no lobby and nothing typed has nowhere to go", () => {
  /* Not an accident to paper over. Before the registry, a build with no lobby
     of its own dialled zero-k.info regardless of which game was on disk. */
  assert.equal(resolveServer(undefined, undefined), undefined);
});
