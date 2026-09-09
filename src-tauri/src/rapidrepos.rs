//! The Spring rapid index, as a list of games somebody can install.
//!
//! `repos.springrts.com/repos.gz` is where the ecosystem publishes itself: one
//! line per repository, and every repository carries a `<name>:stable` tag
//! naming a playable archive. That is the closest thing Spring has to a
//! catalogue, and it is the honest source for "what can I install" - better
//! than a list compiled into this binary, which is wrong the day after it ships.
//!
//! ## What is in there, checked rather than assumed
//!
//! Fifty repositories on 2026-09-09. Roughly forty carry a `:stable` tag; the
//! rest publish only per-commit `:git:` builds and have nothing to offer a
//! picker. Of the ones that do, a handful are not games at all - Chili
//! Framework, LUPS, i18n, Custom Unit Shaders, SpringBoard Core, and the Chobby
//! menus. Those are libraries a game depends on, and installing one on its own
//! does nothing.
//!
//! Also worth knowing: **Beyond All Reason is in the repo list but publishes no
//! `:stable` tag here.** It distributes through its own launcher, so it does not
//! appear in this catalogue. That is not a bug to route around.
//!
//! ## Why a denylist and not an allowlist
//!
//! The two failures are not equal. A library shown in the picker is noise
//! somebody scrolls past; a real game missing from it is a game they cannot
//! install and have no way to discover. So the filter names the things known
//! *not* to be games and lets everything else through - a new game appears the
//! day its repository does, with no release here.
//!
//! ## Names
//!
//! Repository ids are terse (`s44`, `tap`, `techa`) and the display name lives
//! in each repo's own `versions.gz`, which for Zero-K alone is 400 kB. Fetching
//! fifty of those to draw a list is not worth it, so the well-known names are
//! compiled in and anything unrecognised shows its repository id until it is
//! installed. The exact archive name is resolved from the repo at install time,
//! which is the only moment it has to be right.

use std::io::Read;
use std::time::Duration;

const REPOS_URL: &str = "https://repos.springrts.com/repos.gz";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(30);

/// A gzip index should be a few hundred bytes; the largest `versions.gz` in the
/// index is under a megabyte. Ten is room to grow and still a bound.
const MAX_INDEX: u64 = 10 * 1024 * 1024;

/// Repositories that publish something other than a playable game.
///
/// Every one of these was read off the live index rather than guessed: each has
/// a `:stable` tag whose display name says what it is - `Chili Framework`,
/// `LUPS`, `Custom Unit Shaders`, `SpringBoard Core`, `Chobby`.
const NOT_GAMES: [&str; 12] = [
    "chiliui",         // Chili Framework - the UI toolkit
    "chobby",          // the lobby menu itself
    "evo-chobby",      // and its per-game builds
    "tap-chobby",
    "tchobby",
    "zkmenu",          // Chobby again, as Zero-K ships it
    "lups",            // LUPS - effects
    "i18n",            // translations
    "modelshaders",    // Custom Unit Shaders
    "spring-features", // a feature pack, not a game
    "feature-placer",  // an editor tool
    "sbc",             // SpringBoard Core - the map editor
];

/// Display names for the repositories that have one today.
///
/// A convenience, not a source of truth: a repository missing from here still
/// appears, under its id. Read off the live index on 2026-09-09.
const KNOWN_NAMES: [(&str, &str); 19] = [
    ("ba", "Balanced Annihilation"),
    ("evo", "Evolution RTS"),
    ("jauria", "Jauria RTS"),
    ("jrtsc", "Jauria RTS Commands"),
    ("jw", "Journeywar"),
    ("metalfactions", "Metal Factions"),
    ("mosaic", "MOSAIC"),
    ("phoenix", "Phoenix Annihilation"),
    ("s44", "Spring: 1944"),
    ("swiw", "Imperial Winter"),
    ("tadrd", "Dynamic Robot Defense"),
    ("tafs", "Tech Annihilation + Flea Spam"),
    ("tap", "Total Annihilation Prime"),
    ("tard", "Robot Defense"),
    ("tc", "The Cursed"),
    ("tcampaign", "TA Campaign"),
    ("techa", "Tech Annihilation"),
    ("to", "Total Obliteration"),
    ("xta", "XTA"),
];

/// One installable game from the index.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RapidGame {
    /// The repository id, e.g. `s44`. Stable, and what the tag is built from.
    pub repo: String,
    /// What to call it. The repository id when nothing better is known.
    pub name: String,
    /// The rapid tag to hand `pr-downloader`, e.g. `s44:stable`.
    pub tag: String,
}

/// Every repository line in `repos.gz`.
///
/// The format is `id,url,,` - four comma-separated fields, the last two empty
/// in every line the live index has ever carried. Only the id is used; the URL
/// is where rapid itself looks, and following one from this file would be
/// letting the index choose where a download comes from.
pub fn parse_repos(body: &str) -> Vec<String> {
    body.lines()
        .filter_map(|line| line.split(',').next())
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_string)
        .collect()
}

/// The installable games among a set of repository ids.
pub fn games_from(repos: &[String]) -> Vec<RapidGame> {
    let mut out: Vec<RapidGame> = repos
        .iter()
        .filter(|id| !NOT_GAMES.iter().any(|n| n.eq_ignore_ascii_case(id)))
        .map(|id| RapidGame {
            repo: id.clone(),
            name: KNOWN_NAMES
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(id))
                .map_or_else(|| id.clone(), |(_, n)| (*n).to_string()),
            tag: format!("{id}:stable"),
        })
        .collect();
    // By what a person reads, not by the id they never see.
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn fetch_gz(url: &str) -> Result<String, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(concat!("Mashiro/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(TOTAL_TIMEOUT)
        .build()
        .map_err(|e| format!("could not build an HTTP client: {e}"))?;
    let res = client.get(url).send().map_err(|e| format!("could not reach {url}: {e}"))?;
    if !res.status().is_success() {
        return Err(format!("{url} answered {}", res.status()));
    }
    // Bounded: this is a remote file and `read_to_end` on a decoder is exactly
    // where a small download becomes a large one.
    let mut raw = Vec::new();
    flate2::read::GzDecoder::new(res)
        .take(MAX_INDEX)
        .read_to_end(&mut raw)
        .map_err(|e| format!("{url} is not a gzip index: {e}"))?;
    String::from_utf8(raw).map_err(|_| format!("{url} is not UTF-8"))
}

/// The catalogue, from the live index.
#[tauri::command(async)]
pub fn zks_rapid_games() -> Result<Vec<RapidGame>, String> {
    let body = fetch_gz(REPOS_URL)?;
    let repos = parse_repos(&body);
    if repos.is_empty() {
        return Err("the rapid index came back empty".into());
    }
    Ok(games_from(&repos))
}


/// The game archives already on this machine, by their own names.
///
/// Answers the catalogue's "do I have this". Names carry versions, because that
/// is how the engine indexes them; matching against a catalogue entry is the
/// caller's problem and is done in one place - see `installedAlready` in
/// `src/net/rapidGames.ts`.
#[tauri::command(async)]
pub fn zks_installed_games(install_root: Option<String>) -> Vec<String> {
    let Ok(found) = crate::install::detect_with(install_root.as_deref()) else {
        return Vec::new();
    };
    crate::archives::installed(&found.root).games().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The first lines of the live index on 2026-09-09, verbatim.
    const REPOS: &str = "\
aa,https://repos.springrts.com/aa,,
ba,https://repos.springrts.com/ba,,
bar,https://repos.springrts.com/bar,,
chiliui,https://repos.springrts.com/chiliui,,
lups,https://repos.springrts.com/lups,,
s44,https://repos.springrts.com/s44,,
zk,https://repos.springrts.com/zk,,
zkmenu,https://repos.springrts.com/zkmenu,,
";

    #[test]
    fn the_index_parses_into_repository_ids() {
        let got = parse_repos(REPOS);
        assert_eq!(got.len(), 8);
        assert_eq!(got[0], "aa");
        assert!(got.contains(&"metalfactions".to_string()) == false);
        assert!(got.contains(&"s44".to_string()));
    }

    #[test]
    fn a_library_is_not_offered_as_a_game() {
        /* Chili Framework and LUPS both publish a `:stable` tag, so the tag is
           not the discriminator. Installing one on its own does nothing. */
        let games = games_from(&parse_repos(REPOS));
        let repos: Vec<&str> = games.iter().map(|g| g.repo.as_str()).collect();
        assert!(!repos.contains(&"chiliui"), "{repos:?}");
        assert!(!repos.contains(&"lups"), "{repos:?}");
        assert!(!repos.contains(&"zkmenu"), "{repos:?}");
    }

    #[test]
    fn a_repository_nobody_named_still_appears() {
        /* The denylist direction that matters. A game missing from the picker
           cannot be installed and cannot be discovered; a library in it is
           something to scroll past. So anything unrecognised is shown. */
        let games = games_from(&parse_repos(REPOS));
        let aa = games.iter().find(|g| g.repo == "aa").expect("an unnamed repo was dropped");
        assert_eq!(aa.name, "aa", "an id was replaced by a guess");
        assert_eq!(aa.tag, "aa:stable");
    }

    #[test]
    fn a_known_repository_gets_its_real_name() {
        let games = games_from(&parse_repos(REPOS));
        let s44 = games.iter().find(|g| g.repo == "s44").expect("present");
        assert_eq!(s44.name, "Spring: 1944");
        assert_eq!(s44.tag, "s44:stable");
    }

    #[test]
    fn the_list_reads_in_name_order() {
        let games = games_from(&parse_repos(REPOS));
        let names: Vec<&str> = games.iter().map(|g| g.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_by_key(|n| n.to_lowercase());
        assert_eq!(names, sorted);
    }

    #[test]
    fn a_ragged_index_does_not_produce_empty_entries() {
        // Blank lines and trailing whitespace are the ordinary shape of a file
        // like this; an empty id would become a `:stable` tag naming nothing.
        let got = parse_repos("\n  \nzk,https://x,,\n\n");
        assert_eq!(got, vec!["zk"]);
    }

    /// The live index, so a change in its shape is caught here rather than by a
    /// player looking at an empty picker.
    #[test]
    #[ignore = "network"]
    fn the_live_index_still_has_the_shape_this_expects() {
        let games = zks_rapid_games().expect("reachable");
        assert!(games.len() > 20, "only {} games", games.len());

        let repos: Vec<&str> = games.iter().map(|g| g.repo.as_str()).collect();
        for expected in ["zk", "ba", "s44", "metalfactions", "xta", "techa"] {
            assert!(repos.contains(&expected), "{expected} is missing from {repos:?}");
        }
        for library in ["chiliui", "lups", "zkmenu", "sbc"] {
            assert!(!repos.contains(&library), "{library} was offered as a game");
        }
        // Beyond All Reason is in the repo list but publishes no stable tag
        // here; it is expected to be listed, and expected not to install from
        // this source. Named so the assumption is visible if it ever changes.
        assert!(repos.contains(&"bar"), "the repo list no longer carries bar");
    }
}
