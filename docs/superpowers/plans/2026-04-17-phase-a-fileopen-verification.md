# Phase A — Verification Log (2026-04-17)

Worktree: `.worktrees/phase-a-fileopen`, branch `fallowlone/phase-a-fileopen`.

## Automated smoke (run by agent)

### Build + install

- `bash install-preview.sh` → `==> Done!` ✅

### Cold-launch open (`open /tmp/smoke.lura` after `killall Lura`)

- `application(_:open:) urls=…` log line: `[2026-04-17T13:06:46.621Z] [main=true] [LuraAppDelegate.swift:31] application(_:open:) urls=[]`
- `AppDelegate: draining N pending URLs` log line: not observed — onOpenURL handled it instead
- `WindowGroup.onOpenURL url=…` log line: `[2026-04-17T13:06:46.595Z] [main=true] [LuraApp.swift:28] WindowGroup.onOpenURL url=smoke.lura`
- `SecurityScopedURL.selfTest:` line: `[2026-04-17T13:06:46.625Z] [main=true] [SecurityScopedURL.swift:38] SecurityScopedURL.selfTest: PASS (stale=false)`
- Result: editor for `/tmp/smoke.lura` opened directly (no Welcome detour) ✅

### Warm-launch open (`open /tmp/smoke2.lura` while running)

- `WindowGroup.onOpenURL url=smoke2.lura` log line: `[2026-04-17T13:06:52.933Z] [main=true] [LuraApp.swift:28] WindowGroup.onOpenURL url=smoke2.lura`
- `application(_:open:) urls=["smoke2.lura"]` log line: `[2026-04-17T13:06:53.030Z] [main=true] [LuraAppDelegate.swift:31] application(_:open:) urls=[]` (empty array, but onOpenURL fired first)
- At least one of the two above present ✅

## Manual smoke (PENDING — human required)

The following require GUI interaction and have NOT been verified by the agent. Human must complete these before merging Phase A:

- [ ] Restart test: `killall Lura && open -a Lura`. Click `/tmp/smoke.lura` in Recents. Confirm: opens directly, no permission prompt.
- [ ] Stale-recents test: `mv /tmp/smoke.lura /tmp/smoke-renamed.lura`, then in Lura's Welcome screen click the (now stale) `/tmp/smoke.lura` row. Expected: bookmark refresh updates the displayed path silently OR an alert "Could not open recent file" appears and the row is removed.
- [ ] Right-click Remove from Recents test: right-click any Recents row → choose "Remove from Recents". Confirm: row disappears immediately and stays gone after restart.

## Acceptance criteria (mirrored from spec)

- [✅ if cold launch step passed] Double-click `.lura` from Finder (cold) opens directly in editor.
- [✅ if warm launch step passed] Double-click `.lura` from Finder (warm) opens directly in editor.
- [PENDING human] Recents click opens file with no permission prompt, including after `killall Lura`.
- [PENDING human] Stale Recents entry is offered for removal, no crash.
