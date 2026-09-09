/**
 * Which navigation items apply to the game in front of us.
 *
 * Pure, so the rule can be tested without a browser - the same reason
 * `appState.ts` and `playerMenuItems.ts` exist beside their screens.
 *
 * The rule is that an item is gated on the **service it needs**, never on the
 * game's identity. `if (has(game, "codex"))` rather than `if (game.id === "zk")`.
 * The difference matters when the third game arrives: an identity check has to
 * be revisited for every new game, and the one nobody revisits is the one that
 * shows a screen with nothing behind it.
 */
import { has, type Game, type Service } from "../net/games.ts";

export interface NavItem {
  id: string;
  icon: string;
  label: string;
  /** The service this screen talks to. Absent means it works for any game. */
  needs?: Service;
  /** Shown only once there is content behind it, whatever the game. */
  whenInstalled?: boolean;
  whenGalaxy?: boolean;
}

/**
 * The full list, in rail order.
 *
 * Items with no `needs` are the engine's rather than any game's: maps, replays
 * and add-ons work off the data directory and the `.sdfz` format, which every
 * Recoil game shares. That is most of the client, and it is why this is a
 * registry rather than a rewrite.
 */
export const NAV: NavItem[] = [
  { id: "battles", icon: "swords", label: "Battles", needs: "lobby" },
  { id: "chat", icon: "message-square", label: "Chat", needs: "lobby" },
  { id: "queue", icon: "target", label: "Matchmaker", needs: "lobby" },
  { id: "maps", icon: "map", label: "Maps" },
  { id: "codex", icon: "book-marked", label: "Codex", needs: "codex" },
  { id: "friends", icon: "users", label: "Friends", needs: "lobby" },
  { id: "profile", icon: "user", label: "Profile", needs: "profiles" },
  { id: "debrief", icon: "trophy", label: "Last match" },
  { id: "replays", icon: "play", label: "Replays" },
  /* Community missions built in Splaunch, which are a scenario format rather
     than anything a particular game owns - so no service, only the same "is
     there anything behind it" test it always had. */
  { id: "campaigns", icon: "book-open", label: "Campaigns", whenInstalled: true },
  /* The game's own campaign, read out of its files. Two gates, and both are
     needed: the service says the game has one at all, and `hasGalaxy` says this
     machine could actually read it. */
  { id: "galaxy", icon: "globe", label: "Galaxy", needs: "campaign", whenGalaxy: true },
  { id: "apps", icon: "package", label: "Add-ons" },
];

export interface NavContext {
  game?: Game;
  hasCampaigns?: boolean;
  hasGalaxy?: boolean;
  /** The screen being looked at, which is never taken away underfoot. */
  view?: string;
}

/**
 * The items to draw.
 *
 * The current screen always survives, whatever the gates say. Removing the last
 * campaign from Add-ons used to pull the rail out from under somebody standing
 * on it; the same argument now covers a game switch.
 */
export function visibleNav(ctx: NavContext, items: NavItem[] = NAV): NavItem[] {
  return items.filter(n => {
    if (ctx.view && n.id === ctx.view) return true;
    if (n.needs && !has(ctx.game, n.needs)) return false;
    if (n.whenInstalled && !ctx.hasCampaigns) return false;
    if (n.whenGalaxy && !ctx.hasGalaxy) return false;
    return true;
  });
}
