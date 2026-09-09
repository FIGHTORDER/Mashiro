import { test } from "node:test";
import assert from "node:assert/strict";

import { installedAlready, type RapidGame } from "./rapidGames.ts";

const game = (name: string, repo = name.toLowerCase()): RapidGame => ({
  repo,
  name,
  tag: `${repo}:stable`,
});

test("an archive with a version on it is the catalogue's game", () => {
  /* The two sides never spell it the same: the catalogue says
     "Balanced Annihilation", the engine indexes
     "Balanced Annihilation V15.9.8". */
  assert.ok(installedAlready(game("Balanced Annihilation"), [
    "Balanced Annihilation V15.9.8",
  ]));
  assert.ok(installedAlready(game("Zero-K"), ["Zero-K v1.14.8.0"]));
  assert.ok(installedAlready(game("Spring: 1944"), ["Spring: 1944 v3.01"]));
});

test("punctuation and case do not decide it", () => {
  // `Spring: 1944` against `Spring 1944`, and `Zero-K` against `zero-k`.
  assert.ok(installedAlready(game("Spring: 1944"), ["Spring 1944 v3.01"]));
  assert.ok(installedAlready(game("Zero-K"), ["ZERO-K V1.14"]));
});

test("a name that merely appears inside another is not a match", () => {
  /* The reason this is a prefix test and not a substring one. `XTA` is inside
     `Tech Annihilation + Flea Spam`, and a substring test would put an
     Installed badge on XTA for somebody who has never had it. */
  assert.equal(
    installedAlready(game("XTA"), ["Tech Annihilation + Flea Spam v3.8"]),
    false,
  );
  assert.equal(installedAlready(game("Robot Defense"), ["Dynamic Robot Defense v2.35"]), false);
});

test("a game whose name starts another game's name is not confused with it", () => {
  /* Both are real catalogue entries - repositories `techa` and `tafs` - and a
     bare prefix test marked the first installed for anybody who had only the
     second. What follows the name has to look like a version. */
  assert.equal(
    installedAlready(game("Tech Annihilation"), ["Tech Annihilation + Flea Spam v3.8"]),
    false,
  );
  // And the longer one still matches itself.
  assert.ok(installedAlready(
    game("Tech Annihilation + Flea Spam"), ["Tech Annihilation + Flea Spam v3.8"]));
  // As does the shorter one, when it is really there.
  assert.ok(installedAlready(game("Tech Annihilation"), ["Tech Annihilation v4.54"]));
});

test("an archive with no version at all is still itself", () => {
  assert.ok(installedAlready(game("XTA"), ["XTA"]));
});

test("a longer catalogue name is not matched by a shorter archive", () => {
  // Only the archive may carry the extra; a catalogue name never does.
  assert.equal(installedAlready(game("Tech Annihilation"), ["Tech v1"]), false);
});

test("nothing installed marks nothing", () => {
  assert.equal(installedAlready(game("XTA"), []), false);
});

test("an empty catalogue name never matches everything", () => {
  // Folding strips punctuation, so a name of nothing but punctuation folds to
  // "" - and "".startsWith("") is true for every archive there is.
  assert.equal(installedAlready(game("---"), ["Anything At All v1"]), false);
});
