//! How a lobby message becomes bytes, and bytes become messages again.
//!
//! `relay.rs` owns the socket - dialling, keepalive, the shutdown races. None
//! of that is protocol-specific: a socket is a socket. What *is* specific is
//! the framing, and that is the whole of this module.
//!
//! ## Why the framing and not the socket
//!
//! ZkLobbyServer sends `CommandName {json}` separated by newlines; Tachyon
//! sends JSON in WebSocket frames. Those differ in three places - framing,
//! transport and authentication - and only the first is a pure function of the
//! bytes. Putting a trait around the socket loop as well would be an async
//! abstraction over a set of one, which the scope explicitly warns against.
//!
//! This is the seam that is real today: framing is where a message stops and
//! the next begins, it is testable without a network, and it is the part that
//! silently corrupts a session when it is wrong. A second protocol needs a new
//! `LobbyTransport` and a new dialler; it does not need this one rewritten.

/// Turning messages into wire bytes and back.
///
/// `Send + Sync` because the reader and writer halves live on separate tasks.
pub trait LobbyTransport: Send + Sync {
    /// The protocol this speaks, for messages and for the registry check.
    fn name(&self) -> &'static str;

    /// One outgoing message as it goes on the wire.
    fn encode(&self, message: &str) -> Vec<u8>;

    /// Whatever complete messages the newly-arrived bytes complete.
    ///
    /// `pending` carries the partial tail between calls, because a read returns
    /// whatever arrived rather than whole messages: a socket will happily hand
    /// over half a line, and treating that half as a message is how a session
    /// dies on a malformed command nobody sent.
    fn decode(&self, chunk: &[u8], pending: &mut Vec<u8>) -> Vec<String>;
}

/// ZkLobbyServer: UTF-8, newline delimited.
pub struct ZkLines;

impl LobbyTransport for ZkLines {
    fn name(&self) -> &'static str {
        "ZkLobbyServer"
    }

    fn encode(&self, message: &str) -> Vec<u8> {
        let mut out = message.as_bytes().to_vec();
        // Exactly one terminator. A caller that already added it is the
        // ordinary case, and two would send an empty command after every line.
        if !out.ends_with(b"\n") {
            out.push(b'\n');
        }
        out
    }

    fn decode(&self, chunk: &[u8], pending: &mut Vec<u8>) -> Vec<String> {
        pending.extend_from_slice(chunk);
        let mut out = Vec::new();
        // Split on every terminator present, keeping whatever follows the last
        // one for the next call.
        while let Some(at) = pending.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = pending.drain(..=at).collect();
            let text = String::from_utf8_lossy(&line);
            let text = text.trim_end_matches(['\n', '\r']);
            // Blank lines are keepalive noise, not messages.
            if !text.trim().is_empty() {
                out.push(text.to_string());
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_goes_out_with_exactly_one_terminator() {
        assert_eq!(ZkLines.encode("Ping {}"), b"Ping {}\n");
        // Already terminated by the caller, which is the ordinary path.
        assert_eq!(ZkLines.encode("Ping {}\n"), b"Ping {}\n");
    }

    #[test]
    fn a_message_split_across_two_reads_arrives_whole() {
        /* The failure this exists to prevent. A socket returns what arrived,
           not what was sent: treating a half-line as a message hands the lobby
           store a truncated command, and the session dies on something nobody
           sent. */
        let mut pending = Vec::new();
        assert!(ZkLines.decode(b"Welcome {\"Nam", &mut pending).is_empty());
        let got = ZkLines.decode(b"e\":\"zk\"}\n", &mut pending);
        assert_eq!(got, vec![r#"Welcome {"Name":"zk"}"#]);
        assert!(pending.is_empty(), "the tail was not consumed");
    }

    #[test]
    fn several_messages_in_one_read_all_arrive() {
        let mut pending = Vec::new();
        let got = ZkLines.decode(b"A {}\nB {}\nC {}\n", &mut pending);
        assert_eq!(got, vec!["A {}", "B {}", "C {}"]);
    }

    #[test]
    fn a_trailing_partial_is_kept_for_the_next_read() {
        let mut pending = Vec::new();
        let got = ZkLines.decode(b"A {}\nB {", &mut pending);
        assert_eq!(got, vec!["A {}"]);
        assert_eq!(ZkLines.decode(b"}\n", &mut pending), vec!["B {}"]);
    }

    #[test]
    fn blank_lines_are_not_messages() {
        // The server sends them; the store would not know what to do with one.
        let mut pending = Vec::new();
        assert_eq!(ZkLines.decode(b"\n\n  \nA {}\n", &mut pending), vec!["A {}"]);
    }

    #[test]
    fn carriage_returns_do_not_reach_the_parser() {
        /* Nothing sends CRLF today, but a line ending in `\r` parses as a
           command whose JSON has a stray byte on the end, and the failure would
           look like a malformed message rather than a framing bug. */
        let mut pending = Vec::new();
        assert_eq!(ZkLines.decode(b"A {}\r\n", &mut pending), vec!["A {}"]);
    }

    #[test]
    fn invalid_utf8_does_not_take_the_connection_down() {
        // Lossy on purpose: one bad byte should cost one message's fidelity,
        // not the session.
        let mut pending = Vec::new();
        let got = ZkLines.decode(b"A \xff\xfe {}\n", &mut pending);
        assert_eq!(got.len(), 1);
        assert!(got[0].starts_with("A "), "{got:?}");
    }
}
