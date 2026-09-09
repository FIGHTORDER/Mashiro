import { test } from "node:test";
import assert from "node:assert/strict";

import { installChoice, type InstallOption } from "./installChoice.ts";

const CATALOGUE: InstallOption[] = [
  { id: "ba", name: "Balanced Annihilation" },
  { id: "s44", name: "Spring: 1944" },
  { id: "zk", name: "Zero-K" },
];

test("with no catalogue there is still something to install", () => {
  /* The case that matters: the rapid index needs the network and a first launch
     is exactly when it may be missing. A dead button naming nothing would be
     the worst possible first impression. */
  const got = installChoice([], undefined, "zk", "Zero-K");
  assert.equal(got.canChoose, false);
  assert.equal(got.id, "zk");
  assert.equal(got.label, "Zero-K");
});

test("one game is not a choice", () => {
  const got = installChoice([{ id: "zk", name: "Zero-K" }], undefined, "zk");
  assert.equal(got.canChoose, false);
  assert.equal(got.id, "zk");
  assert.equal(got.label, "Zero-K");
});

test("several games are offered, and the default is what is installed first", () => {
  const got = installChoice(CATALOGUE, undefined, "zk");
  assert.equal(got.canChoose, true);
  assert.equal(got.id, "zk");
  assert.equal(got.label, "Zero-K");
});

test("the picked game is what gets installed", () => {
  const got = installChoice(CATALOGUE, "s44", "zk");
  assert.equal(got.id, "s44");
  assert.equal(got.label, "Spring: 1944");
});

test("a selection that is no longer on offer falls back rather than installing nothing", () => {
  /* The catalogue arrives after the dialog opens, so the first selection is a
     guess made before there was anything to guess from - and a stale one must
     not produce a button that installs an id nothing knows. */
  const got = installChoice(CATALOGUE, "techa", "zk");
  assert.equal(got.id, "zk");
  assert.equal(got.label, "Zero-K");
});

test("a default missing from the catalogue still names something installable", () => {
  // The index could omit the default - Beyond All Reason publishes no stable
  // tag, for one - and the button must still say what it will do.
  const got = installChoice([{ id: "ba", name: "Balanced Annihilation" }], undefined, "zk", "Zero-K");
  assert.equal(got.id, "zk");
  assert.equal(got.label, "Zero-K");
});
