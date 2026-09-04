# Restart-safe incomplete campaign index

## Goal

Make failed or interrupted managed builds discoverable and resumable after the installer
restarts, without trusting a frontend path or scanning the filesystem.

## Design

Keep the existing create-once successful-install records unchanged. Add a second append-only
application-data directory, `managed-campaigns/`, containing one create-once start record per
install id. A start record contains only the stable install id, canonical managed root, and
frozen recipe digest.

The orchestrator asks its injected lifecycle dependency to publish this record immediately
after the campaign ledger's record zero has been created or replayed and its identity has been
verified, before preflight or installation work begins. Replaying the same campaign may publish
the identical record; conflicting bytes or another root fail closed.

Discovery reads both registries. A completed-install record wins. Every remaining start record
is checked by opening and replaying the indexed ledger and matching its install id, canonical
root, and frozen recipe digest. A valid ledger without a fresh-copy seal is resumable. A moved,
corrupt, mismatched, or fresh-copy-sealed campaign remains visible but unavailable.

The native bridge resolves `resume_build(install_id)` through that verified projection. The
frontend receives a `resumable` flag and shows one Resume button on the matching Home card.
Neither command accepts an arbitrary path, and no directory tree is searched.

## Testing

- Registry tests prove create-once publication, ledger-backed discovery, identity mismatch,
  moved/corrupt targets, fresh-copy state, and completed-record precedence.
- Orchestrator tests prove the start record hook occurs only after ledger identity is durable.
- Bridge tests recreate the bridge and resume only an indexed, verified install id.
- Frontend tests prove only resumable cards expose Resume and route into the existing build
  event flow.
