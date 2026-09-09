//! springfiles, the content source every Recoil game shares.
//!
//! `pr-downloader` already consults it, so this is not a new dependency for the
//! ecosystem - it is the same index, reached directly, for the fallback path
//! that previously only knew how to ask Zero-K.
//!
//! Three things about the service, all checked against it rather than assumed:
//!
//! * **`springname` is an equality match.** `Comet Catcher Redux` returns the
//!   map; `comet`, `comet*` and `*comet*` all return an empty array. So this
//!   implements [`ContentSource::resolve`] and leaves `search` at its default.
//! * **The category does not need to be sent.** Asking without one searches
//!   everything and the hit says which it is, which is better than guessing
//!   map-or-mod before asking.
//! * **A name it does not know is `[]`, not an error.** HTTP 200, empty array.
//!   Absence is a shape here, exactly as it is in Zero-K's own service.
//!
//! Mirrors come back as `https://` already, but they are filtered to the known
//! host anyway: this list decides what gets downloaded and run, and a mirror
//! field is not ours to trust just because the surrounding JSON was.

use std::time::Duration;

use crate::content_source::{ContentSource, Resolved, ResourceKind};

const ENDPOINT: &str = "https://springfiles.springrts.com/json.php";

/// The only host a mirror may point at.
///
/// Not a general URL check - a specific allowlist, because the alternative is
/// letting a JSON field choose where an archive is fetched from.
const ALLOWED_HOST: &str = "springfiles.springrts.com";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
const TOTAL_TIMEOUT: Duration = Duration::from_secs(30);

pub struct SpringFiles;

impl ContentSource for SpringFiles {
    fn name(&self) -> &'static str {
        "springfiles"
    }

    fn resolve(&self, internal_name: &str) -> Result<Option<Resolved>, String> {
        if internal_name.trim().is_empty() {
            return Ok(None);
        }
        let client = reqwest::blocking::Client::builder()
            .user_agent(concat!("Mashiro/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(TOTAL_TIMEOUT)
            .build()
            .map_err(|e| format!("could not build an HTTP client: {e}"))?;

        let res = client
            .get(format!("{ENDPOINT}?springname={}", percent_encode(internal_name)))
            .send()
            .map_err(|e| format!("could not reach springfiles: {e}"))?;
        if !res.status().is_success() {
            return Err(format!("springfiles answered {}", res.status()));
        }
        let body = res.text().map_err(|e| format!("springfiles sent no body: {e}"))?;
        parse_response(&body)
    }
}


/// Percent-encode one query value.
///
/// Written out rather than pulled in: map names carry spaces and punctuation
/// (`Comet Catcher Redux`, `Supreme-K 3.42`), and this is the only place in
/// this module that needs it. Everything outside the unreserved set goes as
/// %XX, which is always safe even where it was not strictly required.
fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Whether a mirror is somewhere we are willing to fetch from.
///
/// Checked on the host itself rather than with `starts_with`, so that
/// `https://springfiles.springrts.com.evil.example/x.sd7` is refused - it is a
/// different host that merely begins with the right characters.
fn host_allowed(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    // Strip any userinfo: `https://springfiles.springrts.com@evil/x` has host
    // `evil`, and reading up to the first slash would have missed that.
    let host = host.rsplit('@').next().unwrap_or("");
    host.eq_ignore_ascii_case(ALLOWED_HOST)
}

/// The first usable hit in a springfiles response.
///
/// Pure, so the shapes that matter can be tested without the network.
pub fn parse_response(body: &str) -> Result<Option<Resolved>, String> {
    let hits: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("springfiles sent no JSON: {e}"))?;
    let Some(hits) = hits.as_array() else {
        return Err("springfiles sent something that is not a list".into());
    };
    let Some(hit) = hits.first() else {
        // Not an error: the ordinary answer for a name it has never seen.
        return Ok(None);
    };

    let kind = match hit.get("category").and_then(serde_json::Value::as_str) {
        Some("map") => ResourceKind::Map,
        // Anything else it indexes is a playable archive as far as the engine
        // is concerned, and belongs beside the games.
        Some(_) => ResourceKind::Mod,
        None => return Err("a springfiles hit has no category".into()),
    };

    let urls: Vec<String> = hit
        .get("mirrors")
        .and_then(serde_json::Value::as_array)
        .map(|m| {
            m.iter()
                .filter_map(serde_json::Value::as_str)
                .filter(|u| host_allowed(u))
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    let md5 = hit
        .get("md5")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    Ok(Some(Resolved { kind, urls, dependencies: Vec::new(), md5 }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Captured verbatim from the live service on 2026-09-09.
    const COMET: &str = r#"[{"mainQueryTime":0.061256 ,"metaQueriesTime":0.011558 ,
      "jsonTime":0.000050 ,"fid": 980, "name": "Comet Catcher Redux",
      "filename": "comet_catcher_redux.sd7", "path": "maps",
      "md5": "4963ee4d16c0a4490c069ca6a4b629e1",
      "sdp": "269deab3bd402964e09750efb8c225cd", "version": "", "category": "map",
      "size": 4615853, "timestamp": "2011-01-01T17:27:16", "keywords": "land,medium",
      "mirrors": ["https://springfiles.springrts.com/files/maps/comet_catcher_redux.sd7"],
      "tags": [], "springname": "Comet Catcher Redux"}]"#;

    #[test]
    fn a_name_with_spaces_and_punctuation_survives_the_query() {
        // Real map names, which is why this exists at all.
        assert_eq!(percent_encode("Comet Catcher Redux"), "Comet%20Catcher%20Redux");
        assert_eq!(percent_encode("Supreme-K 3.42"), "Supreme-K%203.42");
        // A name is not ours to trust: it arrives from the lobby server.
        assert_eq!(percent_encode("a&b=c"), "a%26b%3Dc");
    }

    #[test]
    fn a_real_response_resolves_to_a_downloadable_map() {
        let got = parse_response(COMET).expect("parses").expect("a hit");
        assert_eq!(got.kind, ResourceKind::Map);
        assert_eq!(got.kind.directory(), "maps");
        assert_eq!(got.md5.as_deref(), Some("4963ee4d16c0a4490c069ca6a4b629e1"));
        assert_eq!(got.urls.len(), 1);
        assert!(got.urls[0].ends_with("comet_catcher_redux.sd7"), "{:?}", got.urls);
    }

    #[test]
    fn a_name_it_has_never_seen_is_absence_and_not_an_error() {
        /* HTTP 200 with an empty array. The distinction matters: reported as an
           error, every name belonging to another game would put a warning in
           front of the player on the ordinary path. */
        assert_eq!(parse_response("[]").expect("parses"), None);
    }

    #[test]
    fn a_mirror_on_another_host_is_refused() {
        /* This list decides what gets downloaded and run. The lookalikes are
           the point - both of these contain the allowed host as a substring. */
        for bad in [
            "https://springfiles.springrts.com.evil.example/x.sd7",
            "https://springfiles.springrts.com@evil.example/x.sd7",
            "http://springfiles.springrts.com/files/maps/x.sd7",
        ] {
            let body = format!(
                r#"[{{"category":"map","md5":"a","mirrors":["{bad}"]}}]"#
            );
            let got = parse_response(&body).expect("parses").expect("a hit");
            assert!(got.urls.is_empty(), "{bad} was accepted");
        }
        assert!(host_allowed("https://springfiles.springrts.com/files/maps/x.sd7"));
    }

    #[test]
    fn a_non_map_lands_beside_the_games() {
        let body = r#"[{"category":"game","md5":"b","mirrors":[]}]"#;
        let got = parse_response(body).expect("parses").expect("a hit");
        assert_eq!(got.kind, ResourceKind::Mod);
        assert_eq!(got.kind.directory(), "games");
    }

    #[test]
    fn a_hit_with_no_category_is_refused_rather_than_guessed() {
        // Guessing map-or-mod decides which directory an archive is written
        // into, and the engine will not find one filed in the wrong place.
        let body = r#"[{"md5":"c","mirrors":[]}]"#;
        assert!(parse_response(body).is_err());
    }

    /// The live service, so a change in its shape is caught here rather than
    /// by a player whose download quietly stopped working.
    #[test]
    #[ignore = "network"]
    fn the_live_service_still_answers_the_shape_this_expects() {
        let got = SpringFiles.resolve("Comet Catcher Redux").expect("reachable");
        let got = got.expect("springfiles still knows this map");
        assert_eq!(got.kind, ResourceKind::Map);
        assert!(!got.urls.is_empty());

        // A map from Beyond All Reason, which is the whole point of this source.
        let bar = SpringFiles.resolve("Red Comet Remake 1.8").expect("reachable");
        assert!(bar.is_some(), "a BAR map did not resolve");

        assert_eq!(SpringFiles.resolve("no such map exists here").unwrap(), None);
    }
}
