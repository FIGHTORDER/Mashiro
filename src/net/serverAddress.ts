/**
 * The lobby server, as somebody types it.
 *
 * Mashiro is not one game's client, so the server cannot be a constant. It has
 * a default - whichever game is installed says where its lobby is, out of the
 * registry in `src-tauri/src/games.rs` - and the login screen lets that be
 * overridden, because a private or test server is a perfectly ordinary thing to
 * point a lobby client at.
 *
 * One field rather than two. `host:port` is how everybody writes a server
 * address, and two boxes where people expect one is a worse trade than parsing
 * a colon.
 */

export interface Server {
  host: string;
  port: number;
}

/** What a field shows for a server. Port omitted when it is the default. */
export function formatServer(server: Server | undefined, defaultPort: number): string {
  if (!server?.host) return "";
  return server.port && server.port !== defaultPort
    ? `${server.host}:${server.port}`
    : server.host;
}

/**
 * Read a typed address.
 *
 * Returns the server, or a reason it cannot be used. A reason rather than a
 * silent fallback: dialling a different machine than the one somebody typed is
 * the one outcome worse than refusing, and a wrong port is how that happens.
 *
 * Empty is not an error - it means "use the default" - and is reported as
 * `undefined` with no reason, which the caller resolves against the game.
 */
export function parseServer(
  text: string,
  defaultPort: number,
): { server?: Server; error?: string } {
  const trimmed = text.trim();
  if (!trimmed) return {};

  /* A scheme is a reasonable thing to paste and a poor thing to guess at: this
     is a raw TCP lobby, not a URL, and quietly dropping `https://` would mean
     dialling port 8200 on a host somebody believed was being fetched over TLS. */
  if (/^[a-z][a-z0-9+.-]*:\/\//i.test(trimmed)) {
    return { error: "Just the host, or host:port - no http:// or wss://." };
  }

  // `[::1]:8200`, and a bare IPv6 address, before the colon rule below.
  const bracketed = trimmed.match(/^\[([^\]]+)\](?::(\d+))?$/);
  if (bracketed) {
    return withPort(bracketed[1], bracketed[2], defaultPort);
  }
  if ((trimmed.match(/:/g) || []).length > 1) {
    return { error: "For an IPv6 address, put it in brackets: [::1]:8200." };
  }

  const [host, port] = trimmed.split(":");
  return withPort(host, port, defaultPort);
}

function withPort(
  host: string,
  port: string | undefined,
  defaultPort: number,
): { server?: Server; error?: string } {
  if (!host) return { error: "That has no host in it." };
  if (/\s/.test(host)) return { error: "A server address has no spaces in it." };
  if (port === undefined || port === "") return { server: { host, port: defaultPort } };

  const n = Number(port);
  if (!Number.isInteger(n) || n < 1 || n > 65535) {
    return { error: `${port} is not a port number.` };
  }
  return { server: { host, port: n } };
}

/**
 * The server to dial: what was typed, else what the game says, else nothing.
 *
 * Nothing is a real answer. A game with no lobby has nowhere to log in, and
 * inventing an address for it would dial somebody else's server on its behalf -
 * which, before the registry existed, is what every build did.
 */
export function resolveServer(
  typed: Server | undefined,
  gameLobby: Server | undefined,
): Server | undefined {
  return typed || gameLobby;
}
