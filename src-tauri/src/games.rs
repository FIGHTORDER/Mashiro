//! Which game this is, as data rather than as code.
//!
//! Mashiro launches Recoil games, and almost everything it does is the same
//! for all of them: a data directory holding `engine/`, `games/` and `maps/`,
//! rapid and springfiles for content, `.sdfz` demos, a TDF start script,
//! `springsettings.cfg`. That machinery is the engine's, not any one game's.
//!
//! What differs between games is a short list of facts - where an installer
//! puts things, which Steam application it is, what its rapid tag is called -
//! and which optional services exist to talk to. Both live here, so that a
//! module doing generic work can ask a question instead of carrying a constant.
//!
//! ## Why a registry and not a trait per game
//!
//! Because most of the difference is data. A trait would need an implementation
//! per game whose body was a list of literals, which is a more elaborate way of
//! writing this table. Traits do appear, but for the parts that are genuinely
//! behaviour - a content search, a battle history - and a game names which
//! implementations it has rather than providing them.
//!
//! ## The rule that keeps this honest
//!
//! A game entry describes; it never decides. Nothing here knows about screens
//! or commands. A feature that only some games have is gated on the *service*
//! it needs being present, so adding a game is adding a row, and adding a
//! feature does not mean revisiting every row.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A game Mashiro knows how to find, fetch and launch.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    /// Stable identifier. Lower case, no spaces - it ends up in paths and
    /// settings keys, so it must not change once anything has stored it.
    pub id: String,
    /// What a person calls it.
    pub name: String,

    /// Directory names an installer is likely to have used, checked under each
    /// of the ordinary install roots. Case is folded when matching, because
    /// Linux does not fold it for us and a `Zero-K` beside a `zero-k` is a real
    /// way to have no install at all.
    pub install_dirs: Vec<String>,
    /// Steam application id, when the game is on Steam.
    pub steam_app_id: Option<u32>,

    /// The rapid tag that names the playable game, e.g. `zk:stable`.
    ///
    /// Used to fetch the game and to recognise its archives. `None` for a game
    /// distributed some other way; content acquisition then needs a name from
    /// the caller rather than a default.
    pub rapid_tag: Option<String>,
    /// How the engine indexes the game archive, before the version.
    ///
    /// Archive names carry a version - `Zero-K v1.14.8.0` - so this is the
    /// prefix a resolver matches on.
    pub archive_prefix: String,

    /// The game's own website, without a trailing slash.
    ///
    /// Where an account is made and where a player is sent for anything the
    /// client does not do itself. Named here because it reaches user-facing
    /// copy: a login screen that says "zero-k.info" to a Beyond All Reason
    /// player is worse than one that says nothing.
    pub site: Option<String>,

    /// Where the lobby server is, and what dialect it speaks.
    ///
    /// `None` for a game with no lobby, which is not the same as an empty host:
    /// absence is what [`Service::Lobby`] is gated on, and a game claiming the
    /// service without an endpoint is a registry error the tests catch.
    pub lobby: Option<Lobby>,

    /// Optional services this game has, by name. See `services`.
    pub services: Vec<Service>,
    /// Integrations that only make sense for this game's own interface.
    pub integrations: Vec<Integration>,
}

/// A lobby server, and the protocol it speaks.
///
/// The host and port used to be a constant in the frontend, which meant every
/// build dialled zero-k.info whatever game was on disk. It belongs here for the
/// same reason every other varying constant does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lobby {
    pub host: String,
    pub port: u16,
    pub protocol: Protocol,
}

/// How a lobby server is spoken to.
///
/// Named rather than assumed, because the transport is not a detail a second
/// game can be slotted into: they differ on framing, on transport and on
/// authentication all at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Protocol {
    /// ZkLobbyServer: line-delimited `CommandName {json}` over plain TCP,
    /// with an MD5 password hash.
    /// What `relay.rs` implements, and the only one this build can speak.
    ZkLobby,
    /// Tachyon: JSON over a WebSocket with OAuth 2, used by Beyond All Reason.
    /// Named so a game can describe itself honestly; nothing speaks it yet, and
    /// a game declaring it does not get [`Service::Lobby`].
    Tachyon,
}

/// A network service a game may or may not have.
///
/// The point of naming them is that a feature can be gated on the service it
/// needs rather than on the game's identity: `if game.has(Service::Profiles)`
/// rather than `if game.id == "zk"`. Adding a game then cannot silently enable
/// a screen that has nothing to talk to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Service {
    /// A lobby server: battle rooms, chat, matchmaking.
    Lobby,
    /// Searchable content beyond what rapid and springfiles reach.
    ContentSearch,
    /// Past battles and downloadable replays.
    BattleHistory,
    /// Player profiles, ranks and awards.
    Profiles,
    /// A unit database worth browsing in the client.
    Codex,
    /// A single-player campaign the client can read and launch.
    Campaign,
}

/// A way of reaching into the game's own interface.
///
/// Separate from `Service` because these are local file surgery rather than
/// network calls, and because they are the least portable thing here: each one
/// exists because a specific game's interface is built a specific way. Nothing
/// abstracts over them - a game either has the integration or does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Integration {
    /// Community widgets into `LuaUI/Widgets`, enabled through the game's own
    /// config. Depends on the game having a widget handler that reads one.
    Widgets,
    /// Skins for the game's in-game interface.
    UiSkins,
    /// A loading screen the client can replace.
    LoadScreen,
    /// An in-game button that raises the lobby.
    LobbyButton,
}

impl Game {
    pub fn has(&self, s: Service) -> bool {
        self.services.contains(&s)
    }

    pub fn integrates(&self, i: Integration) -> bool {
        self.integrations.contains(&i)
    }

    /// Does this directory name look like one of ours?
    ///
    /// Folded, because the engine's own VFS folds case and a person typing a
    /// path does too. `install.rs` matches directory entries against this.
    pub fn matches_dir(&self, name: &str) -> bool {
        self.install_dirs.iter().any(|d| d.eq_ignore_ascii_case(name))
    }

    /// The install directories to probe under `root`, in the order given.
    pub fn dirs_under(&self, root: &Path) -> Vec<PathBuf> {
        self.install_dirs.iter().map(|d| root.join(d)).collect()
    }
}

/// A game found on disk that the curated table does not describe.
///
/// The Spring ecosystem has dozens of games - the rapid index alone lists
/// around forty, and anyone can build another - so a table with a row per game
/// is a list that is wrong the day after it ships. A game nobody wrote a row
/// for is still perfectly playable: the data directory, rapid, springfiles,
/// `.sdfz` demos and the start script are the engine's, and that is most of
/// this client.
///
/// So an unrecognised install becomes a game named after what is in it, with no
/// services and no integrations. Everything generic works; everything that
/// needs a service the registry cannot vouch for stays hidden.
///
/// Named after the directory, which is the one thing every install has and is
/// what the person who made it chose to call the game. Reading the display name
/// out of the archives would be more precise and is not available in general -
/// a data directory that has been made but not filled has `engine/` and nothing
/// else - so this takes the answer that is always there rather than the answer
/// that is sometimes better.
pub fn discovered(dir_name: &str) -> Game {
    let name = display_name(dir_name);
    Game {
        id: format!("discovered:{}", name.to_ascii_lowercase()),
        name: name.clone(),
        install_dirs: vec![dir_name.to_string()],
        steam_app_id: None,
        rapid_tag: None,
        archive_prefix: name,
        site: None,
        lobby: None,
        services: Vec::new(),
        integrations: Vec::new(),
    }
}

/// An archive name with its version taken off.
///
/// Archives are indexed as `Balanced Annihilation V12.1` or `Evolution RTS
/// v16.00`, and the part before the version is what a person calls the game.
/// Split on the last space when what follows looks like a version, so a name
/// that simply ends in a word - `Journeywar` - is left whole.
pub fn display_name(archive: &str) -> String {
    let trimmed = archive.trim();
    match trimmed.rsplit_once(' ') {
        Some((head, tail)) if looks_like_version(tail) && !head.is_empty() => head.to_string(),
        _ => trimmed.to_string(),
    }
}

fn looks_like_version(s: &str) -> bool {
    let body = s.strip_prefix(['v', 'V']).unwrap_or(s);
    !body.is_empty() && body.starts_with(|c: char| c.is_ascii_digit())
}

/// Which game an install directory holds, if the registry recognises it.
///
/// By the directory's own name, because that is the one thing every install
/// has before anything has been read out of it. The archives inside would be a
/// better answer and are not always there - a freshly made data directory has
/// `engine/` and nothing else - so this is deliberately the cheap check.
///
/// A directory the curated table does not describe is **not** `None`: it is a
/// [`discovered`] game named after itself, with no services and no
/// integrations.
///
/// That distinction is the whole of multi-game support, and getting it wrong
/// was a real bug. `None` sent every caller to `default_game`, which is Zero-K
/// and carries every service - so a Balanced Annihilation install was
/// identified as Zero-K, shown Zero-K's Profile and Codex screens, and had its
/// player names sent to zero-k.info. The comment here used to claim the
/// fallback left services hidden. It did the opposite.
///
/// `None` now means only "there is no directory name to read", which is a
/// broken path rather than an unknown game.
pub fn identify(root: &Path) -> Option<Game> {
    let name = root.file_name()?.to_str()?;
    if let Some(curated) = all().into_iter().find(|g| g.matches_dir(name)) {
        return Some(curated);
    }
    // A shared data directory is not a game, however many it holds.
    if is_shared_data_dir(name) {
        return None;
    }
    Some(discovered(name))
}

/// Is this the engine's own data directory rather than one game's?
///
/// `~/.config/spring` and `~/.spring` hold whatever the machine has - several
/// games, or none - so there is no single game to name after them. Calling one
/// "spring" would put a game in the picker that nobody installed and offer to
/// launch it.
///
/// A short list rather than a rule, because these are conventions with names,
/// not a pattern: anything else is somebody's install of something.
fn is_shared_data_dir(name: &str) -> bool {
    const SHARED: [&str; 3] = ["spring", ".spring", ".config"];
    SHARED.iter().any(|s| s.eq_ignore_ascii_case(name))
}

/// Every game this build knows about.
///
/// Compiled in rather than fetched. A registry read from a URL would be a way
/// for whoever serves it to point an installer at a directory of their
/// choosing, and the list changes about as often as the client ships.
pub fn all() -> Vec<Game> {
    vec![zero_k(), beyond_all_reason()]
}

/// A game by id, or `None` if this build does not know it.
pub fn by_id(id: &str) -> Option<Game> {
    all().into_iter().find(|g| g.id == id)
}

/// The one to use when nothing has been chosen.
///
/// Zero-K, because it is the game this client grew up on and the only one whose
/// every service is implemented. Not a statement that it is the important one -
/// a default has to be something, and an arbitrary one would be worse.
pub fn default_game() -> Game {
    zero_k()
}

fn zero_k() -> Game {
    Game {
        id: "zk".into(),
        name: "Zero-K".into(),
        /* `zk` is the directory this client makes for itself - see
           `managed::root` - and it is the ordinary path for a new player, who
           has the game downloaded into it rather than finding one. Left out, a
           managed install is identified as a game called "zk" with no services
           and the lobby, codex and campaign all disappear. */
        install_dirs: vec!["Zero-K".into(), "zk".into()],
        steam_app_id: Some(334_920),
        rapid_tag: Some("zk:stable".into()),
        archive_prefix: "Zero-K".into(),
        site: Some("https://zero-k.info".into()),
        lobby: Some(Lobby {
            host: "zero-k.info".into(),
            port: 8200,
            protocol: Protocol::ZkLobby,
        }),
        services: vec![
            Service::Lobby,
            Service::ContentSearch,
            Service::BattleHistory,
            Service::Profiles,
            Service::Codex,
            Service::Campaign,
        ],
        integrations: vec![
            Integration::Widgets,
            Integration::UiSkins,
            Integration::LoadScreen,
            Integration::LobbyButton,
        ],
    }
}

fn beyond_all_reason() -> Game {
    /* Content and launching, not the lobby.
     *
     * BAR's lobby speaks Tachyon - JSON over a WebSocket, OAuth 2, its own
     * generated schema - which shares nothing with the line-delimited TCP this
     * client's transport was built for. That is a second transport and an
     * authentication flow, and it is deliberately not in this scope; `services`
     * says so by omission rather than by a comment somewhere else.
     *
     * Everything else works today, because it was never Zero-K's to begin with:
     * the data directory layout, rapid, springfiles, pr-downloader, `.sdfz`
     * demos and the start script are the engine's. */
    Game {
        id: "bar".into(),
        name: "Beyond All Reason".into(),
        install_dirs: vec![
            "Beyond All Reason".into(),
            "Beyond-All-Reason".into(),
            "BeyondAllReason".into(),
        ],
        steam_app_id: Some(1_905_610),
        rapid_tag: Some("byar:test".into()),
        archive_prefix: "Beyond All Reason".into(),
        site: Some("https://www.beyondallreason.info".into()),
        /* Described but not offered: the endpoint is real and the protocol is
           named, and `services` still omits `Lobby` because nothing here can
           speak Tachyon. Writing it down is what makes the gap visible rather
           than making the game look like it has no lobby at all. */
        lobby: Some(Lobby {
            host: "server.beyondallreason.info".into(),
            port: 443,
            protocol: Protocol::Tachyon,
        }),
        services: vec![],
        integrations: vec![],
    }
}

// ------------------------------------------------------------- commands ---

/// Every game this build knows about, for a picker.
#[tauri::command]
pub fn zks_games() -> Vec<Game> {
    all()
}

/// The game the interface should present itself as.
///
/// Resolved from the install that was actually found rather than from a
/// setting, so the screens on offer match the files on disk. An install we do
/// not recognise - a shared Spring data directory, somebody else's game -
/// falls back to the default rather than erroring: there is still an engine
/// there to launch, and a client showing no screens at all would be a worse
/// answer than one showing the ones it cannot prove are wrong.
#[tauri::command(async)]
pub fn zks_active_game(install_root: Option<String>) -> Game {
    crate::install::detect_with(install_root.as_deref())
        .ok()
        .and_then(|found| identify(&found.root))
        .unwrap_or_else(default_game)
}

/// Can this build actually talk to that game's lobby?
///
/// Separate from [`Service::Lobby`] on purpose. The service says the game has a
/// lobby worth showing screens for; this says the transport in this binary can
/// speak to it. A game can have the first without the second - Beyond All
/// Reason does - and conflating them is how a client offers a login box that
/// can never succeed.
pub fn lobby_speakable(game: &Game) -> bool {
    matches!(game.lobby.as_ref().map(|l| l.protocol), Some(Protocol::ZkLobby))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_game_nobody_wrote_a_row_for_is_still_itself() {
        /* The bug this replaces: an unrecognised install identified as `None`,
           every caller fell back to `default_game` - Zero-K, which carries
           every service - and a Balanced Annihilation player was shown Zero-K's
           Profile screen and had their name sent to zero-k.info. */
        let root = std::path::Path::new("/games/Balanced Annihilation");
        let got = identify(root).expect("a directory with a name is a game");

        assert_eq!(got.name, "Balanced Annihilation");
        assert_ne!(got.id, "zk", "an unknown game was identified as Zero-K");
        assert!(got.services.is_empty(), "services were invented for it");
        assert!(got.integrations.is_empty(), "integrations were invented for it");
        assert!(got.lobby.is_none(), "it was given somebody else's lobby");
    }

    #[test]
    fn a_curated_game_still_wins_over_discovery() {
        let got = identify(std::path::Path::new("/games/Zero-K")).expect("identified");
        assert_eq!(got.id, "zk");
        assert!(got.has(Service::Lobby), "the curated row was lost");
    }

    #[test]
    fn discovery_is_stable_for_the_same_directory() {
        // The id ends up in settings keys, so it must not move under a player.
        let a = discovered("Evolution RTS");
        let b = discovered("Evolution RTS");
        assert_eq!(a.id, b.id);
        assert!(a.matches_dir("evolution rts"), "case was not folded on the way back");
    }

    #[test]
    fn a_version_on_the_end_is_not_part_of_the_name() {
        /* Spring archives and install directories both carry versions. A player
           should see "Metal Factions", not "Metal Factions v1.2.3". */
        assert_eq!(display_name("Metal Factions v1.2.3"), "Metal Factions");
        assert_eq!(display_name("Balanced Annihilation V12.1"), "Balanced Annihilation");
        // A name that merely ends in a word keeps all of it.
        assert_eq!(display_name("Journeywar"), "Journeywar");
        assert_eq!(display_name("Tech Annihilation"), "Tech Annihilation");
        // And nothing is trimmed down to nothing.
        assert_eq!(display_name("v1.0"), "v1.0");
    }

    #[test]
    fn a_discovered_game_gets_the_generic_client_and_nothing_more() {
        /* What "support as many Spring games as possible" actually means: the
           engine's half works for everything, and only the parts that need a
           service somebody vouched for are withheld. */
        let g = discovered("Spring 1944");
        assert!(!g.install_dirs.is_empty(), "it cannot be found again");
        assert!(!g.archive_prefix.is_empty(), "a start script could not name it");
        for service in [
            Service::Lobby, Service::ContentSearch, Service::BattleHistory,
            Service::Profiles, Service::Codex, Service::Campaign,
        ] {
            assert!(!g.has(service), "{service:?} was assumed for an unknown game");
        }
    }

    #[test]
    fn a_game_offering_the_lobby_has_one_this_build_can_speak() {
        /* The registry describes; this stops it describing something that
           cannot work. Claiming `Lobby` with no endpoint, or with a protocol
           nothing here implements, would put a login screen in front of a
           player and fail every attempt. */
        for game in all() {
            if game.has(Service::Lobby) {
                let lobby = game.lobby.as_ref()
                    .unwrap_or_else(|| panic!("{} offers a lobby with no endpoint", game.id));
                assert!(!lobby.host.is_empty(), "{} has an empty lobby host", game.id);
                assert!(lobby.port > 0, "{} has no lobby port", game.id);
                assert!(
                    lobby_speakable(&game),
                    "{} offers a lobby this build cannot speak ({:?})",
                    game.id, lobby.protocol
                );
            }
        }
    }

    #[test]
    fn a_lobby_we_cannot_speak_is_not_offered_as_a_service() {
        // Beyond All Reason: the endpoint is written down, Tachyon is named,
        // and the service is withheld. Described honestly, offered never.
        let bar = by_id("bar").expect("bar is registered");
        assert_eq!(bar.lobby.as_ref().map(|l| l.protocol), Some(Protocol::Tachyon));
        assert!(!lobby_speakable(&bar));
        assert!(!bar.has(Service::Lobby), "a lobby nothing can speak was offered");
    }

    #[test]
    fn an_install_directory_names_its_game() {
        assert_eq!(identify(Path::new("/games/Zero-K")).map(|g| g.id), Some("zk".into()));
        assert_eq!(
            identify(Path::new("/steamapps/common/Beyond All Reason")).map(|g| g.id),
            Some("bar".into())
        );
        // Folded, for the same reason `matches_dir` folds.
        assert_eq!(identify(Path::new("/games/zero-k")).map(|g| g.id), Some("zk".into()));
    }

    #[test]
    fn a_shared_data_directory_is_not_a_game() {
        /* It holds whatever the machine has - several games, or none - so there
           is nothing to name after it. Naming one "spring" would put a game in
           the picker that nobody installed.

           This used to also cover any unrecognised directory, back when an
           unknown game was `None` and the caller fell back to Zero-K. Now an
           unknown game is itself, so only the shared directories are excluded -
           see `a_game_nobody_wrote_a_row_for_is_still_itself`. */
        assert!(identify(Path::new("/home/me/.config/spring")).is_none());
        assert!(identify(Path::new("/home/me/.spring")).is_none());
        assert!(identify(Path::new("/")).is_none());
    }

    #[test]
    fn somebody_elses_game_is_theirs_rather_than_ours() {
        // Identified, and given nothing it did not earn.
        let got = identify(Path::new("/games/Total Annihilation")).expect("a game");
        assert_eq!(got.name, "Total Annihilation");
        assert!(got.services.is_empty());
    }

    #[test]
    fn a_site_is_a_bare_https_origin() {
        /* It reaches user-facing copy and outbound links. A trailing slash
           doubles up wherever a path is appended, and http would be a downgrade
           on a link somebody is about to type a password after. */
        for g in all() {
            let Some(site) = g.site.as_deref() else { continue };
            assert!(site.starts_with("https://"), "{}: {site}", g.id);
            assert!(!site.ends_with('/'), "{}: {site} has a trailing slash", g.id);
        }
    }

    #[test]
    fn ids_are_unique_and_path_safe() {
        /* Ids reach settings keys and directory names, so a duplicate or a
           space is a bug that shows up far from here. */
        let games = all();
        let mut ids: Vec<&str> = games.iter().map(|g| g.id.as_str()).collect();
        ids.sort_unstable();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count, "two games share an id");

        for g in &games {
            assert!(!g.id.is_empty(), "{} has no id", g.name);
            assert!(
                g.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-'),
                "{} is not path-safe",
                g.id
            );
            assert!(!g.name.is_empty(), "{} has no name", g.id);
            assert!(!g.install_dirs.is_empty(), "{} names no install directory", g.id);
            assert!(!g.archive_prefix.is_empty(), "{} names no archive prefix", g.id);
        }
    }

    #[test]
    fn a_game_is_found_by_its_id_and_nothing_else_is() {
        assert_eq!(by_id("zk").map(|g| g.name), Some("Zero-K".to_string()));
        assert_eq!(by_id("bar").map(|g| g.name), Some("Beyond All Reason".to_string()));
        assert!(by_id("").is_none());
        assert!(by_id("ZK").is_none(), "ids are lower case, and matching is exact");
        assert!(by_id("../../etc").is_none());
    }

    #[test]
    fn directory_matching_folds_case() {
        /* Windows folds case for you and Linux does not, which is how a Linux
           user ends up with an install the client cannot see. */
        let zk = by_id("zk").unwrap();
        assert!(zk.matches_dir("Zero-K"));
        assert!(zk.matches_dir("zero-k"));
        assert!(zk.matches_dir("ZERO-K"));
        assert!(!zk.matches_dir("Zero-K-Infrastructure"), "a prefix is not a match");

        // BAR ships under several spellings depending on where it came from.
        let bar = by_id("bar").unwrap();
        for spelling in ["Beyond All Reason", "beyond-all-reason", "BeyondAllReason"] {
            assert!(bar.matches_dir(spelling), "{spelling} is not recognised");
        }
    }

    #[test]
    fn a_game_without_a_service_does_not_claim_it() {
        /* The whole point of the registry: a feature asks whether the service
           exists, so adding a game cannot light up a screen with nothing behind
           it. BAR has no lobby here because Tachyon is out of scope. */
        let bar = by_id("bar").unwrap();
        assert!(!bar.has(Service::Lobby), "BAR would need Tachyon");
        assert!(!bar.has(Service::Profiles));
        assert!(!bar.integrates(Integration::Widgets), "ZK_data.lua is Zero-K's own");

        let zk = by_id("zk").unwrap();
        assert!(zk.has(Service::Lobby));
        assert!(zk.integrates(Integration::Widgets));
    }

    #[test]
    fn every_integration_belongs_to_a_game_that_could_use_it() {
        /* An integration writes into the game's own directories. Claiming one
           for a game with no install directory to write into would be a crash
           waiting for the first person who installs that game. */
        for g in all() {
            if !g.integrations.is_empty() {
                assert!(
                    !g.install_dirs.is_empty(),
                    "{} claims integrations but names nowhere to put them",
                    g.id
                );
            }
        }
    }

    #[test]
    fn the_default_is_a_game_the_registry_knows() {
        let d = default_game();
        assert_eq!(by_id(&d.id), Some(d));
    }
}

#[cfg(test)]
mod managed_root_tests {
    use super::*;

    #[test]
    fn the_managed_install_directory_is_recognised_as_zero_k() {
        /* `managed::root` puts the install this client makes itself at
           `<app data>/zk`, and that is the ordinary path for a new player: the
           client downloads Zero-K into it. Identified as anything else, the
           lobby, codex and campaign all disappear for the default install. */
        let got = identify(std::path::Path::new("/appdata/io.github.fightorder.mashiro/zk"))
            .expect("a directory with a name is a game");
        assert_eq!(got.id, "zk", "the managed install is not recognised: {}", got.name);
        assert!(got.has(Service::Lobby), "the default install lost its lobby");
    }
}

#[cfg(test)]
mod managed_layout_tests {
    use super::*;

    /// Mirrors the sanitising in `managed::root_for`, which is what actually
    /// builds the path. Kept here because the property being checked is about
    /// the *ids*: whatever the registry calls a game has to survive becoming a
    /// directory name, and two games must never land in the same one.
    fn dir_for(id: &str) -> String {
        id.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
            .collect()
    }

    #[test]
    fn zero_ks_managed_directory_is_unchanged_by_going_per_game() {
        /* The migration argument: the single managed directory used to be `zk`,
           and Zero-K's registry id is `zk`, so an install made before this keeps
           its exact path and nothing has to move. */
        assert_eq!(dir_for(&default_game().id), "zk");
        assert_eq!(dir_for(&by_id("zk").unwrap().id), "zk");
    }

    #[test]
    fn no_two_games_share_a_managed_directory() {
        /* The bug this replaces: everything downloaded into `zk`, so Balanced
           Annihilation installed by the client was then identified as Zero-K and
           offered Zero-K's lobby and codex. */
        let mut seen = std::collections::HashSet::new();
        for game in all() {
            let dir = dir_for(&game.id);
            assert!(!dir.is_empty(), "{} has no id to name a directory after", game.name);
            assert!(seen.insert(dir.clone()), "{} collides on {dir}", game.name);
        }
    }

    #[test]
    fn a_discovered_games_id_cannot_escape_the_data_directory() {
        /* A discovered id is built from a directory name found on disk, so it is
           not ours and must not be able to climb out of the folder it names. */
        for hostile in ["../../etc", "a/b", "..", r"C:\windows", "with space"] {
            let dir = dir_for(&discovered(hostile).id);
            assert!(!dir.contains('/') && !dir.contains('\\'), "{hostile} -> {dir}");
            assert!(!dir.contains(".."), "{hostile} -> {dir}");
            assert!(!dir.is_empty(), "{hostile} -> empty");
        }
    }
}
