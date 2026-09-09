//! Zero-K, as one game's adapter rather than as the shape of the client.
//!
//! Everything under here speaks something only Zero-K speaks: the HTML of
//! zero-k.info, its SOAP content service, Chobby's campaign Lua, the custom-key
//! encoding its start scripts use. None of it is a Recoil convention, and none
//! of it is reachable except through a trait in `content_source`, `services` or
//! `transport` - each of which asks the registry first.
//!
//! The rule that keeps the split meaningful: **nothing at the crate root may
//! name a module in here.** The root is the engine's - install layout, rapid,
//! `.sdfz` demos, start scripts, the socket - and it talks to games through
//! traits. A second game is a sibling directory, not an edit to the root.
//!
//! Zero-K support stays because it is the point; it just lives somewhere that
//! says whose it is.

pub mod battles;
pub mod campaignpack;
pub mod campaignscript;
pub mod content;
pub mod customkey;
pub mod galaxy;
pub mod web;
