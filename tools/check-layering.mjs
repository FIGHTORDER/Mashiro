#!/usr/bin/env node
/**
 * The crate root is the engine's; `src-tauri/src/zk/` is one game's.
 *
 * The separation is only worth having if it holds, and it erodes one
 * convenient import at a time: a root module reaches into `zk` for a type that
 * is nearly right, and six months later the "generic" core only compiles
 * because Zero-K is there. This is the check that stops that.
 *
 * The rule: **no module at the crate root may name `zk`.** The root talks to
 * games through `content_source`, `services` and `transport`; a game's adapter
 * is chosen by the registry, behind those traits.
 *
 * Two files are allowed to name it, because choosing the implementation is
 * exactly their job. They are listed rather than pattern-matched, so adding a
 * third is a decision somebody makes on purpose.
 *
 *   node tools/check-layering.mjs
 */
import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SRC = join(ROOT, "src-tauri", "src");

/** The seams. Each picks a game's implementation from the registry. */
const WIRING = new Set([
  "content_source.rs", // chain() -> ZeroKContent
  "services.rs",       // profiles()/battles() -> ZeroKWeb / ZeroKBattles
  "lib.rs",            // the command registry, which must name every handler
]);

/* `content.rs` reaches Zero-K's downloader directly for the file-name and
   fetch helpers. Recorded as a known exception rather than quietly allowed:
   it is the one place the split is still leaky, and naming it here is what
   makes it a task instead of a habit. */
const KNOWN_LEAKS = new Set(["content.rs"]);

const offenders = [];
for (const name of readdirSync(SRC)) {
  if (!name.endsWith(".rs")) continue;
  if (WIRING.has(name) || KNOWN_LEAKS.has(name)) continue;
  const body = readFileSync(join(SRC, name), "utf8");
  // Comments are allowed to discuss Zero-K; code may not reach into it.
  const code = body
    .split("\n")
    .filter(l => !/^\s*(\/\/|\*|\/\*)/.test(l))
    .join("\n");
  const hits = [...code.matchAll(/\bcrate::zk::\w+|(?<![:\w])zk::\w+/g)].map(m => m[0]);
  if (hits.length) offenders.push(`  ${name}: ${[...new Set(hits)].join(", ")}`);
}

if (offenders.length) {
  console.error("A module in the engine core reaches into a game's adapter:\n");
  console.error(offenders.join("\n"));
  console.error(
    "\nThe core talks to games through content_source, services and transport."
    + "\nIf this really is wiring, add the file to WIRING in tools/check-layering.mjs"
    + "\nand say why.",
  );
  process.exit(1);
}

const leaks = [...KNOWN_LEAKS].filter(n => readdirSync(SRC).includes(n));
console.log(
  `core does not reach into zk/ (${WIRING.size} wiring files, ${leaks.length} known leak`
  + `${leaks.length === 1 ? "" : "s"}: ${leaks.join(", ") || "none"})`,
);
