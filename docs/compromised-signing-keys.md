# Compromised issuer signing keys

The issuer registry records an immutable, commitment-only status when an issuer
signing key is known to be compromised. Only the registry administrator may
create this record. The contract never accepts or stores raw public or private
key material.

`mark_signing_key_compromised` accepts an issuer identifier, key hash,
algorithm identifier, and reason hash. The key must already be known as the
issuer's active, pending, or retained historical key. A successful call records
version `1`, the current ledger, and a per-issuer sequence number. Replaying the
same declaration or declaring an unknown key fails atomically.

If the key is active or pending, that entry is removed in the same transaction.
The immutable marker remains authoritative: the same commitment cannot later be
proposed or activated. Historical key records retain their original encoding;
the compromise record is an additive sidecar and therefore requires no legacy
record migration.

`get_compromised_signing_key` performs a direct status lookup.
`get_compromised_key_history` returns records in sequence order
using a zero-based cursor and a limit from 1 through 20. A cursor at or beyond
the current count returns an empty page. Zero and oversized limits fail with
`IssuerError::BatchTooLarge`.
