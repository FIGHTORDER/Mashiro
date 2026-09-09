//! The game services a screen talks to, and the guard that keeps them honest.
//!
//! `content_source` covers acquiring content, which every Recoil game needs.
//! These two are different: they are a game's own website, and a game either
//! runs one or does not. Zero-K implements both by scraping zero-k.info;
//! Beyond All Reason has its own site with a different shape, and a bare Recoil
//! game has none at all.
//!
//! ## Why a trait over a set of one
//!
//! Not for the polymorphism - there is one implementation each and no second
//! one queued up. It is for the **gate**. Before this, the commands behind these
//! screens took no install root and so could not tell which game was in front of
//! them: a Beyond All Reason player whose nav rail correctly hid the Profile
//! screen could still reach the command underneath it, and it would go and ask
//! zero-k.info about them. The rail was the only thing standing between another
//! game's player and a request to Zero-K's servers.
//!
//! So the trait exists to give [`profiles`] and [`battles`] something to return
//! `None` *from*. A screen that is gated in the interface is now also gated
//! underneath it, and the two gates read from the same registry.
//!
//! ## The vocabulary stays where it is
//!
//! `Profile`, `ArchivePage` and the rest keep living in `zkweb` and `zkbattles`
//! rather than moving here. They are the shape of Zero-K's site today, and
//! inventing a neutral shape for them now would be guessing at what a second
//! site's answer looks like. The second implementation is what should force
//! that, and until there is one a neutral type would be Zero-K's type wearing a
//! different name.

use crate::games::{self, Game, Service};

/// A game's player profiles.
///
/// `Send` because the scraping runs on a blocking worker rather than the thread
/// drawing the window, and the handle has to travel there.
pub trait PlayerProfiles: Send {
    fn profile(&self, who: &str) -> Result<Option<crate::zk::web::Profile>, String>;
    fn ratings(&self, account_id: u32, category: u8)
        -> Result<Vec<crate::zk::web::RatingPoint>, String>;
}

/// A game's record of past battles. `Send` for the same reason as above.
pub trait BattleHistory: Send {
    fn search(&self, query: &crate::zk::battles::BattleQuery) -> crate::zk::battles::ArchivePage;
    fn lookup_players(&self, term: &str) -> Vec<crate::zk::battles::PlayerMatch>;
    fn download_replay(
        &self,
        id: u64,
        install_root: Option<&str>,
    ) -> Result<std::path::PathBuf, String>;
}

/// The game on disk, by the same route every screen uses.
///
/// An install we cannot identify falls back to the default rather than to
/// nothing: there is still an engine there, and refusing every service on a
/// shared Spring data directory would be a worse answer than offering the ones
/// we cannot prove are wrong.
fn game_for(install_root: Option<&str>) -> Game {
    crate::install::detect_with(install_root)
        .ok()
        .and_then(|found| games::identify(&found.root))
        .unwrap_or_else(games::default_game)
}

/// The profile service for this install, or `None` if the game has none.
pub fn profiles(install_root: Option<&str>) -> Option<Box<dyn PlayerProfiles>> {
    let game = game_for(install_root);
    if !game.has(Service::Profiles) {
        return None;
    }
    // One implementation, and it is Zero-K's site. A game that claimed the
    // service without one would be a registry error, not something to guess at.
    (game.id == "zk").then(|| Box::new(crate::zk::web::ZeroKWeb) as Box<dyn PlayerProfiles>)
}

/// The battle archive for this install, or `None` if the game has none.
pub fn battles(install_root: Option<&str>) -> Option<Box<dyn BattleHistory>> {
    let game = game_for(install_root);
    if !game.has(Service::BattleHistory) {
        return None;
    }
    (game.id == "zk").then(|| Box::new(crate::zk::battles::ZeroKBattles) as Box<dyn BattleHistory>)
}

/// Does the game on this install have a campaign this client can read?
///
/// The campaign reader speaks Chobby's format - `planetDefs.lua`, `VFS.Include`,
/// the whole `campaign/sample` tree. That is Zero-K's interface architecture,
/// not a Recoil convention, so the reader is gated rather than generalised:
/// there is no second campaign format to abstract over, and inventing one from
/// a sample of one would be guessing.
///
/// Gated here as well as in the rail because the rail hides the screen and this
/// stops the commands underneath it running Lua out of some other game's files.
pub fn has_campaign(install_root: Option<&str>) -> bool {
    game_for(install_root).has(Service::Campaign)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_game_claiming_a_service_has_something_behind_it() {
        /* The registry describes; this is the check that it does not describe
           something that does not exist. A game listing `Profiles` with no
           implementation would show the screen and then fail on every lookup,
           which is worse than not offering it. */
        for game in games::all() {
            if game.has(Service::Profiles) {
                assert_eq!(
                    game.id, "zk",
                    "{} claims Profiles but nothing implements it",
                    game.id
                );
            }
            if game.has(Service::BattleHistory) {
                assert_eq!(
                    game.id, "zk",
                    "{} claims BattleHistory but nothing implements it",
                    game.id
                );
            }
        }
    }

    #[test]
    fn beyond_all_reason_is_not_offered_zero_ks_website() {
        // The whole point of the guard: BAR has its own site, and asking
        // zero-k.info about a BAR player is a request that should never leave.
        let bar = games::by_id("bar").expect("bar is registered");
        assert!(!bar.has(Service::Profiles), "BAR would be sent to zero-k.info");
        assert!(!bar.has(Service::BattleHistory), "BAR would be sent to zero-k.info");
    }
}
