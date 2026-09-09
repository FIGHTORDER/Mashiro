/**
 * What the first-run dialog offers to install.
 *
 * Pure, so the rule can be tested without a browser - the sibling of
 * `navItems.ts` and `addonKinds.ts`, and here for the same reason: the dialog
 * itself only appears after a login, so anything decided inside it is awkward
 * to reach from a test and easy to get quietly wrong.
 *
 * The rule it exists to hold: **there is always something to install.** The
 * catalogue is the rapid index, which needs the network, and a first launch is
 * exactly when that might be missing. An empty list must produce a working
 * button naming the default, not a dead one naming nothing.
 */

/** A game that can be installed. `id` is the rapid repository id. */
export interface InstallOption {
  id: string;
  name: string;
}

export interface InstallChoice {
  /** Whether to draw a picker at all. */
  canChoose: boolean;
  /** The game the button will install. Never undefined. */
  id: string;
  /** What the button says it will install. */
  label: string;
}

/**
 * Resolve the selection against what is actually on offer.
 *
 * `selected` is whatever the picker last set, which can name a game that is no
 * longer in the list - the catalogue arrives after the dialog opens, so the
 * first value is a guess made before there was anything to guess from.
 */
export function installChoice(
  games: InstallOption[],
  selected: string | undefined,
  defaultId: string,
  defaultName = "a game",
): InstallChoice {
  const chosen = games.find(g => g.id === selected)
    || games.find(g => g.id === defaultId);
  return {
    // One entry is not a choice, and none is not a picker.
    canChoose: games.length > 1,
    id: chosen?.id || defaultId,
    label: chosen?.name || defaultName,
  };
}
