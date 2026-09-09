/**
 * Bridge to the Rust TCP relay. The relay owns the socket; this module owns
 * nothing but the invoke/listen plumbing.
 */
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type RelayStatus =
  | { kind: "connecting" }
  | { kind: "connected" }
  | { kind: "disconnected"; reason: string };

/**
 * Where to connect, when nobody has said.
 *
 * There is no such place. Mashiro is not one game's client: the lobby comes
 * from the installed game's registry entry, or from what somebody typed on the
 * login screen, and a build-time constant here meant every install dialled
 * zero-k.info whatever game was on disk - including games with a lobby of their
 * own, and games with none at all.
 *
 * So the host is required, and a caller that has not resolved one is a bug
 * rather than a Zero-K user.
 */
export function connect(host: string, port: number): Promise<void> {
  if (!host) throw new Error("No lobby server to connect to.");
  return invoke("zks_connect", { host, port });
}

export function sendLine(line: string): Promise<void> {
  return invoke("zks_send", { line });
}

export function disconnect(): Promise<void> {
  return invoke("zks_disconnect");
}

/** base64(raw md5 digest) - computed in Rust so we do not hand-roll MD5. */
export function passwordHash(password: string): Promise<string> {
  return invoke("zks_password_hash", { password });
}

export function onLine(cb: (line: string) => void): Promise<UnlistenFn> {
  return listen<string>("zks://line", e => cb(e.payload));
}

export function onStatus(cb: (s: RelayStatus) => void): Promise<UnlistenFn> {
  return listen<RelayStatus>("zks://status", e => cb(e.payload));
}

/** True when running inside the Tauri shell rather than a plain browser tab. */
export function inTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}
