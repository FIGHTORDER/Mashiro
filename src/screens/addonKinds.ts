/**
 * Which add-on kinds apply to the game in front of us.
 *
 * The sibling of `navItems.ts`, and pure for the same reason. The rail hides
 * whole screens; this hides the kinds inside Add-ons, which is where the
 * game-specific integrations live.
 *
 * The distinction that decides each row: **whose interface does it modify?**
 *
 * - Apps and Shiro skins are the client's own. Every game gets them.
 * - Campaign packs are a scenario format, not any one game's - the same
 *   argument `navItems.ts` makes for the Campaigns rail item.
 * - Widgets, game UI skins and loading screens reach into the *game's* own
 *   interface. `ZK_data.lua` and Chili are Zero-K's architecture; writing them
 *   into a Beyond All Reason install would place files nothing there reads.
 *
 * That last group is gated on an `Integration` rather than a `Service`, because
 * these are local file surgery rather than network calls - and unlike a service,
 * there is nothing to abstract over. A game either has the integration or does
 * not, which is exactly what the scope calls for: gate them and move on.
 */
import { integrates, type Game, type Integration } from "../net/games.ts";

export interface AddonKind {
  id: string;
  icon: string;
  label: string;
  /** The game integration this kind writes to. Absent means it is ours. */
  needs?: Integration;
}

/** The full list, in rail order. */
export const KINDS: AddonKind[] = [
  /* Ungated on purpose: the rapid index is the ecosystem's, not any one
     game's, and a player with any Recoil install can fetch any Spring game
     from it. This is the one kind that is about games rather than add-ons, and
     it earns its place here because it is where somebody goes to get one. */
  { id: "games", icon: "gamepad-2", label: "Games" },
  { id: "apps", icon: "package", label: "Apps" },
  { id: "skins", icon: "palette", label: "Client skins" },
  { id: "loadscreens", icon: "image", label: "Loading screens", needs: "loadScreen" },
  { id: "widgets", icon: "puzzle", label: "Widgets", needs: "widgets" },
  { id: "uiskins", icon: "monitor", label: "Game UI skins", needs: "uiSkins" },
  { id: "campaign", icon: "book-open", label: "Campaign" },
];

/**
 * The kinds to draw.
 *
 * The kind being looked at always survives, whatever the gates say, for the
 * same reason the rail keeps the current screen: taking the ground out from
 * under somebody standing on it is worse than showing one row too many.
 */
export function visibleKinds(
  game: Game | undefined,
  current?: string,
  kinds: AddonKind[] = KINDS,
): AddonKind[] {
  return kinds.filter(k => {
    if (current && k.id === current) return true;
    return !k.needs || integrates(game, k.needs);
  });
}
