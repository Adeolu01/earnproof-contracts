# Versioned subject nullifier domains

Subject nullifiers are opaque 32-byte commitments. Proof registry never stores
the raw subject identifier. Every stored `SubjectNullifier` binds the
commitment to a nonzero `domain_version`, preventing clients from interpreting
commitments produced by different derivation algorithms as the same domain.

Protocol config initializes version 1 as supported. Governance can enable or
disable versions with `set_nullifier_domain_version`; the mutation requires the
admin, advances `config_version`, and is included in bounded config history.
Clients can use `is_nullifier_domain_supported` or the bounded
`get_nullifier_domain_versions` query. At most 16 versions can be indexed.

Version 1 is exactly the existing `earnproof.subject-pseudonym.v1` byte
derivation. `compute_subject_nullifier_commitment(..., 1, ...)` delegates to
the legacy helper, so every existing v1 commitment remains unchanged. Versions
2 and later hash the canonical Soroban XDR encoding of:

```text
("earnproof_subject_nullifier", domain_version, issuer, purpose, subject)
```

`register_proof_with_nullifier` validates a nonzero version and commitment,
checks current protocol support, performs the normal proof registration, and
stores the versioned nullifier in the same invocation. Any validation,
authorization, stale-policy, replay, or capacity failure rolls the invocation
back atomically. Disabling a version affects new registrations only; existing
proof bindings remain readable and byte-stable.
