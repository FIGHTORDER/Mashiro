//! Turning a content name into something downloadable, whatever the game.
//!
//! `pr-downloader` reaches most things through rapid and springfiles, and what
//! it cannot reach is what this covers. Zero-K runs its own service with the
//! community maps and every custom mod; other Recoil games do not, and before
//! this the fallback path asked Zero-K's service about names that were never
//! going to be in it.
//!
//! ## The shape the sources actually have
//!
//! Two capabilities, deliberately separate, because the sources differ on
//! exactly this line:
//!
//! * **resolve** - an exact name to a URL. Both sources do it.
//! * **search** - a substring to a list of candidates. Zero-K's service does
//!   it; springfiles does not. Its `springname` parameter is an equality match:
//!   `Comet Catcher Redux` returns the map, and `comet`, `comet*` and `*comet*`
//!   all return nothing. Checked against the live service rather than assumed,
//!   because a search box wired to a source that cannot search would look
//!   like a service outage rather than a design mistake.
//!
//! So `search` has a default that returns nothing, and a caller offering a
//! search box asks the registry for [`Service::ContentSearch`] first.
//!
//! ## Order
//!
//! [`chain`] puts the game's own service first and springfiles second: the
//! game's own is the one that knows about its own content, and springfiles is
//! the ecosystem-wide floor that every Recoil game shares.

use crate::games::{Game, Service};

/// What a resolved name turns out to be.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceKind {
    Map,
    Mod,
}

impl ResourceKind {
    /// Where it belongs under the data directory.
    pub fn directory(self) -> &'static str {
        match self {
            ResourceKind::Map => "maps",
            ResourceKind::Mod => "games",
        }
    }
}

/// A name resolved to somewhere it can be fetched from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolved {
    pub kind: ResourceKind,
    /// Downloadable URLs, https, allowed host only. May be empty, which is a
    /// real answer: the source knows the name but does not serve the file.
    pub urls: Vec<String>,
    /// Other internal names this one needs. May include `rapid://` entries,
    /// which are pr-downloader's job rather than ours.
    pub dependencies: Vec<String>,
    /// The archive's MD5, where the source publishes one.
    pub md5: Option<String>,
}

/// One hit from a content search.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MapHit {
    pub name: String,
    /// How the source rates it. Zero-K says `MatchMaker` for the curated set
    /// the ladder draws from; another source may say nothing at all.
    pub support: String,
    /// Addresses the map's page on the source's own site, where it has one.
    pub resource_id: Option<u32>,
    /// Absent, not false: a source that sends no flag has not told us the
    /// answer, which is a different claim from "no".
    pub is_1v1: Option<bool>,
    pub is_teams: Option<bool>,
    pub is_ffa: Option<bool>,
    pub is_special: Option<bool>,
}

/// Somewhere content can be looked up.
pub trait ContentSource {
    /// For messages. A failure has to say whose failure it was, because "the
    /// content service" means nothing to a player with two of them.
    fn name(&self) -> &'static str;

    /// An exact internal name to something downloadable.
    ///
    /// `Ok(None)` means the source has never heard of it, which is not an
    /// error - it is the ordinary answer for a name belonging to another game.
    fn resolve(&self, internal_name: &str) -> Result<Option<Resolved>, String>;

    /// A substring to candidates, best first.
    ///
    /// Defaulted to nothing because most sources cannot do this. A source that
    /// can should also be named in its game's [`Service::ContentSearch`], so
    /// the interface knows whether to offer a search box at all.
    fn search(&self, _query: &str) -> Result<Vec<MapHit>, String> {
        Ok(Vec::new())
    }
}

/// The sources to try for this game, in order.
///
/// The game's own service first where it has one, springfiles always, because
/// it is the floor the whole Recoil ecosystem shares.
pub fn chain(game: &Game) -> Vec<Box<dyn ContentSource>> {
    let mut out: Vec<Box<dyn ContentSource>> = Vec::new();
    if game.has(Service::ContentSearch) && game.id == "zk" {
        out.push(Box::new(crate::zk::content::ZeroKContent));
    }
    out.push(Box::new(crate::springfiles::SpringFiles));
    out
}

/// Ask each source in turn, stopping at the first that knows the name.
///
/// An error from one source is not fatal: it is reported and the next is
/// tried, because one service being down is not a reason to fail a download
/// another could have served.
pub fn resolve_any(
    sources: &[Box<dyn ContentSource>],
    name: &str,
    mut note: impl FnMut(String),
) -> Option<Resolved> {
    for source in sources {
        match source.resolve(name) {
            Ok(Some(r)) => return Some(r),
            Ok(None) => {}
            Err(e) => note(format!("{}: {e}", source.name())),
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Silent;
    impl ContentSource for Silent {
        fn name(&self) -> &'static str {
            "silent"
        }
        fn resolve(&self, _: &str) -> Result<Option<Resolved>, String> {
            Ok(None)
        }
    }

    struct Broken;
    impl ContentSource for Broken {
        fn name(&self) -> &'static str {
            "broken"
        }
        fn resolve(&self, _: &str) -> Result<Option<Resolved>, String> {
            Err("unreachable".into())
        }
    }

    struct Knows;
    impl ContentSource for Knows {
        fn name(&self) -> &'static str {
            "knows"
        }
        fn resolve(&self, _: &str) -> Result<Option<Resolved>, String> {
            Ok(Some(Resolved {
                kind: ResourceKind::Map,
                urls: vec!["https://example/x.sd7".into()],
                dependencies: Vec::new(),
                md5: None,
            }))
        }
    }

    #[test]
    fn a_source_that_is_down_does_not_stop_the_next_one() {
        /* One service being unreachable is not a reason to fail a download
           another source could have served, and the player is told which one
           failed rather than "the content service". */
        let sources: Vec<Box<dyn ContentSource>> = vec![Box::new(Broken), Box::new(Knows)];
        let mut said = Vec::new();
        let got = resolve_any(&sources, "anything", |m| said.push(m));
        assert!(got.is_some(), "the working source was never asked");
        assert_eq!(said.len(), 1);
        assert!(said[0].starts_with("broken:"), "{:?}", said);
    }

    #[test]
    fn a_name_nobody_knows_is_not_an_error() {
        // The ordinary answer for a name belonging to another game.
        let sources: Vec<Box<dyn ContentSource>> = vec![Box::new(Silent)];
        let mut said = Vec::new();
        assert!(resolve_any(&sources, "nope", |m| said.push(m)).is_none());
        assert!(said.is_empty(), "silence was reported as a failure: {said:?}");
    }

    #[test]
    fn search_is_opt_in() {
        // The default exists so a source that cannot search says so quietly
        // rather than every caller having to know which ones can.
        assert!(Silent.search("comet").unwrap().is_empty());
    }

    #[test]
    fn every_game_gets_springfiles_and_only_zero_k_gets_its_own_service() {
        for game in crate::games::all() {
            let names: Vec<&str> = chain(&game).iter().map(|s| s.name()).collect();
            assert!(
                names.contains(&"springfiles"),
                "{} has no ecosystem-wide source: {names:?}",
                game.id
            );
            if game.id != "zk" {
                assert!(
                    !names.iter().any(|n| n.contains("Zero-K")),
                    "{} was pointed at Zero-K's service: {names:?}",
                    game.id
                );
            }
        }
    }
}
