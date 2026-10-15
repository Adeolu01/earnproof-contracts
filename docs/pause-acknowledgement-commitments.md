# Pause acknowledgement commitments

Protocol config and proof registry can append commitment-only evidence that an
authorized responder reviewed an active pause. Acknowledgement is deliberately
separate from pause control: it never changes, clears, or extends a pause.

The administrator governs incident operators with `set_pause_acknowledger`.
The administrator and enabled incident operators may call `acknowledge_pause`
for an active scope. The record stores version `1`, a monotonically increasing
contract-local sequence, the scope, the fixed-size acknowledgement hash, the
public actor address and role, and the current ledger. No raw incident report,
memo, credential, or private response data is accepted.

An all-zero hash, an inactive pause, an unauthorized actor, sequence exhaustion,
or reuse of a previously recorded hash fails atomically. A proof-registry
acknowledgement accepts either its local scoped pause or the effective pause
reported by protocol config. Existing pause state is authoritative throughout.

`get_pause_acknowledgement` performs a direct hash lookup.
`get_pause_acknowledgements` reads the append-only sequence with a zero-based
cursor and a limit from 1 through 20. A cursor at or beyond the current count
returns an empty page. These additive sidecars do not change legacy pause
storage and require no migration.
