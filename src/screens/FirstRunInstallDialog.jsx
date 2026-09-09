import React from "react";
import { Dialog, Button, Icon, Select } from "../ds/shiro.js";

import { installChoice } from "./installChoice.ts";

/* Shown once, the first time Mashiro gets far enough to know whether there is a
 * game on the machine.
 *
 * It exists because both answers are worth saying out loud. Somebody who
 * already has a game should be told they do not need to download it again - a
 * launcher that stays silent about an existing install invites a second copy.
 * Somebody who does not have one should be offered the choice here, rather than
 * having to find it in Settings, which is where it lived and where nobody
 * looked.
 *
 * ## Why the game is chosen here
 *
 * Mashiro launches any Recoil game, so "install it for you" has to ask which.
 * Choosing silently would mean somebody who came for Balanced Annihilation
 * downloading a gigabyte of Zero-K to find out it was the wrong one.
 *
 * The list is the rapid index, which is the ecosystem's own catalogue - see
 * `src-tauri/src/rapidrepos.rs`. Reading it needs the network, and a first
 * launch is exactly when that might be missing, so `games` may be empty and
 * this still has to work: with nothing to choose between it installs the
 * default rather than showing an empty picker.
 *
 * Deliberately not a wizard. There are two facts and at most three choices, and
 * the one that downloads a gigabyte is a deliberate press rather than something
 * somebody defaults their way into. */
export default function FirstRunInstallDialog({
  open, install, engine, root, games = [], defaultGameId = "zk",
  onInstall, onSettings, onClose,
}) {
  const found = Boolean(install);
  const [choice, setChoice] = React.useState(defaultGameId);

  /* The rule lives in `installChoice.ts` so it can be tested: this dialog only
     appears after a login, which makes anything decided inside it awkward to
     reach and easy to get quietly wrong. */
  const { canChoose, id: chosenId, label } = installChoice(games, choice, defaultGameId, "Zero-K");

  return (
    <Dialog
      open={open}
      title={found ? "A game is already here" : "No game installed yet"}
      width={460}
      onClose={onClose}
      footer={found ? (
        <Button variant="primary" onClick={onClose}>Good to go</Button>
      ) : (
        <>
          <Button variant="ghost" onClick={onClose}>Not now</Button>
          <Button variant="quiet" onClick={() => { onClose(); onSettings?.(); }}>
            I have one elsewhere
          </Button>
          <Button variant="primary" disabled={!engine}
            onClick={() => { onClose(); onInstall?.(chosenId); }}>
            Install {label}
          </Button>
        </>
      )}
    >
      <div style={{ display: "flex", flexDirection: "column", gap: "var(--sp-5)" }}>
        {found ? (
          <>
            <div style={{ display: "flex", gap: "var(--sp-4)", alignItems: "flex-start" }}>
              <Icon name="check" size={16}
                style={{ color: "var(--signal-ok, var(--text-hi))", marginTop: 2 }} />
              <span style={{ font: "var(--text-ui-sm)", color: "var(--text-body)",
                lineHeight: 1.5 }}>
                Mashiro found your installation, so there is nothing to download.
                Games launch straight into it.
              </span>
            </div>
            <div style={{ display: "grid", gridTemplateColumns: "auto 1fr",
              gap: "var(--sp-3) var(--sp-5)" }}>
              <span className="lab">FOUND VIA</span>
              <span style={{ font: "var(--text-ui-sm)", color: "var(--text-body)" }}>
                {install.source}
              </span>
              <span className="lab">PATH</span>
              <span style={{ font: "var(--w-regular) var(--size-tiny)/1.4 var(--font-mono)",
                color: "var(--text-body)", overflowWrap: "anywhere" }}>{install.root}</span>
            </div>
          </>
        ) : (
          <>
            <span style={{ font: "var(--text-ui-sm)", color: "var(--text-body)",
              lineHeight: 1.5 }}>
              Mashiro can install one for you - the engine first, then the game,
              then maps as battles need them. Nothing is shared with a Steam
              copy, so this is a separate installation of about a gigabyte.
            </span>

            {canChoose && (
              <label style={{ display: "grid", gap: "var(--sp-2)" }}>
                <span className="lab">GAME</span>
                <Select value={choice} onChange={e => setChoice(e.target.value)}>
                  {games.map(g => (
                    <option key={g.id} value={g.id}>{g.name}</option>
                  ))}
                </Select>
              </label>
            )}

            {root && (
              <div style={{ display: "grid", gridTemplateColumns: "auto 1fr",
                gap: "var(--sp-3) var(--sp-5)" }}>
                <span className="lab">WOULD GO IN</span>
                <span style={{ font: "var(--w-regular) var(--size-tiny)/1.4 var(--font-mono)",
                  color: "var(--text-body)", overflowWrap: "anywhere" }}>{root}</span>
              </div>
            )}

            {/* Said here rather than discovered later: if they already own one,
                pointing Mashiro at it is cheaper than downloading it twice. */}
            <span style={{ font: "var(--text-ui-sm)", color: "var(--text-low)",
              lineHeight: 1.5 }}>
              Already have a game somewhere Mashiro did not look? Point it at the
              folder instead - it is the same game either way.
            </span>
          </>
        )}
      </div>
    </Dialog>
  );
}
