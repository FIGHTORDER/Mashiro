//! Mints a Zero-K Steam auth ticket and prints it, then exits.
//!
//! Run by `shiro`'s `steam.rs` when somebody chooses to sign in with Steam.
//! Writes exactly one line to stdout and nothing else:
//!
//! ```text
//! ok <hex>
//! err <sentence>
//! ```
//!
//! ## What the ticket is
//!
//! `GetAuthSessionTicket`, hex encoded, which is precisely what Zero-K's own
//! client sends as `SteamAuthToken` (`ChobbyLauncher/SteamClient.cs`). The
//! lobby server hands it to Steam's `ISteamUserAuth/AuthenticateUserTicket`
//! along with Zero-K's app id and its own web API key, gets a `steamid` back,
//! and looks up the account. Nothing here talks to Steam's web API and nothing
//! here needs a key.
//!
//! ## Why it initialises as app 334920
//!
//! A ticket is only valid for the app it was minted for, and the server checks
//! it against Zero-K's id. So this has to introduce itself to Steam as Zero-K.
//! That is the same thing any third-party client would have to do, and it is
//! the part of this worth asking the Zero-K developers about rather than
//! assuming - which is why it lives in a process that exists for one second
//! instead of for as long as the launcher is open.
//!
//! Ownership is what Steam checks, not installation: an account that has ever
//! added Zero-K can mint a ticket without the game on disk.
//!
//! ## Handling
//!
//! The ticket is a credential. It goes to stdout, is read once by the parent,
//! and is never written to a log or a file. It is single use and expires on
//! its own in minutes, but that is a reason not to be careless rather than a
//! reason to be.

use std::io::Write;

use steamworks::networking_types::NetworkingIdentity;
use steamworks::Client;

/// Which Steam application to ask for a ticket as.
///
/// A ticket is scoped to one app, so this has to be the game whose lobby the
/// ticket is being presented to - it is worthless for any other. It used to be
/// a constant, which is the same as saying the client only ever signs in to one
/// game; the caller now names the app and the registry knows the numbers.
///
/// Zero-K when nothing says otherwise, so an older caller that passes no
/// argument keeps working rather than failing in a way that reads as Steam
/// being broken.
const DEFAULT_APP_ID: u32 = 334_920;

/// The app id to use, from `--app-id <n>` or the default.
///
/// Parsed rather than trusted: a value that is not a number is a caller bug,
/// and starting Steam as the wrong application would produce a ticket the
/// lobby rejects with no useful message.
fn app_id_from(args: &[String]) -> Result<u32, String> {
    let mut it = args.iter();
    while let Some(a) = it.next() {
        if a == "--app-id" {
            let Some(v) = it.next() else {
                return Err("--app-id was given with no number after it".into());
            };
            return v.parse().map_err(|_| format!("--app-id {v} is not a number"));
        }
    }
    Ok(DEFAULT_APP_ID)
}

/// How long to pump callbacks waiting for the ticket to become usable.
///
/// `GetAuthSessionTicket` hands back its bytes immediately but they are not
/// valid until Steam answers with `GetAuthSessionTicketResponse`, and that
/// answer only arrives while callbacks are being run. Sending too early gets
/// the ticket rejected, which surfaces to a player as an unexplained failed
/// login, so this waits.
const CALLBACK_WAIT: std::time::Duration = std::time::Duration::from_millis(2500);
const CALLBACK_STEP: std::time::Duration = std::time::Duration::from_millis(20);

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let line = match app_id_from(&args).and_then(ticket) {
        Ok(hex) => format!("ok {hex}"),
        Err(why) => format!("err {why}"),
    };
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    if line.starts_with("err ") {
        std::process::exit(1);
    }
}

fn ticket(app_id: u32) -> Result<String, String> {
    /* Steam not running, or an account that does not own the game, both land
       here. Neither is an error worth alarming anybody about: it means this
       machine cannot sign in with Steam, and the password still can. */
    let client = Client::init_app(app_id).map_err(|e| {
        format!(
            "Steam is not available ({e}). Is Steam running, and is the game in your library?"
        )
    })?;

    let user = client.user();
    if user.steam_id().raw() == 0 {
        return Err("Steam did not report an account.".into());
    }

    let (_handle, bytes) = user.authentication_session_ticket(NetworkingIdentity::new());
    if bytes.is_empty() {
        return Err("Steam returned an empty ticket.".into());
    }

    let mut waited = std::time::Duration::ZERO;
    while waited < CALLBACK_WAIT {
        client.run_callbacks();
        std::thread::sleep(CALLBACK_STEP);
        waited += CALLBACK_STEP;
    }

    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_app_id_comes_from_the_caller() {
        /* A ticket is scoped to one Steam application. Hardcoding the number
           meant this could only ever sign in to Zero-K, which is the whole
           thing being taken apart. */
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        assert_eq!(app_id_from(&args(&["--app-id", "1905610"])), Ok(1_905_610));
        assert_eq!(app_id_from(&args(&["--app-id", "334920"])), Ok(334_920));
        // Nothing given: the old behaviour, so an older caller still works.
        assert_eq!(app_id_from(&args(&[])), Ok(DEFAULT_APP_ID));
        assert_eq!(app_id_from(&args(&["--other", "x"])), Ok(DEFAULT_APP_ID));
    }

    #[test]
    fn a_malformed_app_id_is_refused_rather_than_guessed() {
        /* Falling back to the default here would hand back a ticket for the
           wrong game, which the lobby rejects with nothing to explain why. */
        let args = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();

        assert!(app_id_from(&args(&["--app-id", "zero-k"])).is_err());
        assert!(app_id_from(&args(&["--app-id", "-1"])).is_err());
        assert!(app_id_from(&args(&["--app-id"])).is_err());
    }
}
