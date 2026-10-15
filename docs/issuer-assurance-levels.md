# Issuer Assurance Level Commitments

Issuer registry governance may publish a compact assurance tier with
`set_issuer_assurance_level`. Version 1 accepts tiers 1 through 255. The value
is a public policy identifier, not accreditation evidence: no certificate,
auditor report, personal data, or other preimage is accepted by the contract.

Each accepted change receives the transaction ledger as its exact effective
boundary and a checked monotonic per-issuer sequence. Repeating the current
version and tier is rejected without a write. Unknown issuers, revoked issuers,
unsupported versions, tier zero, and sequence overflow also fail atomically.

`get_issuer_assurance_level` returns the current commitment.
`get_issuer_assurance_at_sequence` resolves a retained immutable binding, and
`get_issuer_assurance_history` returns at most 16 retained changes in ascending
sequence order. Once the ring advances beyond a sequence, direct lookup returns
`None`; proofs that bind a sequence retain their original proof fields and are
never rewritten by a later assurance change.
