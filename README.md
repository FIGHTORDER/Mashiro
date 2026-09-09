# Mashiro

A lobby client for [Recoil](https://github.com/beyond-all-reason/RecoilEngine) games.

Mashiro finds the games already on your machine, fetches what a match needs, and
launches the engine. Zero-K is supported in full; Beyond All Reason and other
Recoil games are supported as far as their own services allow.

## What works for which game

Almost everything a lobby client does belongs to the engine rather than to any
one game: a data directory holding `engine/`, `games/` and `maps/`, rapid and
springfiles for content, `.sdfz` demos, a TDF start script, `springsettings.cfg`.
That machinery is shared.

What differs is which *services* a game runs, so a feature is gated on the
service it needs rather than on the game's name. A game with no profile service
does not show a profile screen; it does not show a broken one.

| | Zero-K | Beyond All Reason | Any other Spring game |
|---|---|---|---|
| Install detection, launch | yes | yes | yes |
| Content via rapid / springfiles | yes | yes | yes |
| Replays, maps, add-ons, settings | yes | yes | yes |
| Map search | yes | - | - |
| Lobby, chat, matchmaking | yes | - | - |
| Profiles, battle history | yes | - | - |
| Campaign, unit codex | yes | - | - |
| Widgets, game UI skins, loading screen | yes | - | - |

**A game does not need a row to work.** The Spring ecosystem has dozens of games
- the rapid index alone lists around forty - so an install the registry does not
recognise is identified as itself rather than guessed at, and gets everything in
the top three rows. Balanced Annihilation, Evolution RTS, Metal Factions,
Spring 1944, Tech Annihilation, XTA and anything else you have installed are all
launchable today with no code change.

A row in `src-tauri/src/games.rs` adds the things that cannot be inferred: a
Steam application id, the directory names an installer uses, and which network
services the game runs. It describes and never decides - a feature is gated on
the *service* it needs, never on the game's name, so adding a game cannot
silently switch on a screen with nothing behind it.

## Building

Node 22+ and a Rust toolchain. `claudedoc/SHIRO-BUILDING.md` has the platform
prerequisites - the Linux build has two that fail late in the bundle rather than
at the start.

```
npm ci
npm run tauri dev
```

## Checks

```
npm test && npm run check:names && npm run typecheck && npm run build
npm run test:demo && npm run test:e2e
cargo test
```

## Relationship to Shiro

Mashiro is a game-agnostic fork of [Shiro](https://github.com/FIGHTORDER/shiro),
which is Zero-K-only by design. The scope and the order of work are in
`claudedoc/SCOPE-MASHIRO.md`.
