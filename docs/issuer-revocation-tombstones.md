# Issuer revocation tombstones

Issuer revocation is terminal. When an issuer is revoked, issuer-registry
writes a version-1 persistent tombstone in the same invocation as the issuer
status update. The tombstone contains only:

- `version`
- `issuer_id_hash`
- `revoked_at`
- `revoked_ledger`

It deliberately excludes issuer addresses, metadata, provenance, policy
commitments, signing keys, and the revocation reason. A tombstone is checked
before registration and therefore prevents reuse of a revoked identifier even
when the full issuer record is archived or unavailable. Replaying revocation
does not rewrite its original timestamp or ledger.

`get_issuer_revocation_tombstone` returns `Unknown`, `NotRevoked`, or `Revoked`
without conflating a missing issuer with a known non-revoked issuer.
`get_issuer_revocation_statuses` preserves request order and duplicates and
rejects more than 50 identifiers before reading storage. Both a successful
direct query and `keepalive_revocation_tombstone` extend the persistent entry's
TTL. Operators must keep tombstones alive under the same archival policy used
for other durable registry evidence.
