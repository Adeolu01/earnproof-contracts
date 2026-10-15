# Issuer Signing-Key Lifecycle Sequences

Every accepted issuer signing-key proposal, activation, retirement, and
compromise receives a monotonically increasing sequence scoped to that issuer.
Rotation produces two ordered transitions: the prior active key is retired,
then the pending key is activated. Both sequence values are reserved with
checked arithmetic before either transition is written.

Lifecycle data is an additive sidecar, so the persisted
`SigningKeyCommitment` shape remains compatible. Each record contains only a
key hash, algorithm identifier, transition, sequence, and effective ledger.
No private key material or incident narrative is accepted.

`get_signing_key_usage_sequence` returns the latest committed value.
`get_issuer_signing_key_state` returns the latest state of one commitment, and
`get_signing_key_lifecycle` returns at most 20 entries from the most recent 32
transitions. Stale cursors clamp to the oldest retained entry. Failed,
unauthorized, replayed, unknown-key, and overflow transitions do not advance
the sequence. A compromised commitment is terminal and cannot be proposed
again.
