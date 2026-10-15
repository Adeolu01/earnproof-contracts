# Versioned schema policy digests

`get_schema_policy_digest(schema_version)` returns a compact commitment with
the digest encoding version, current protocol `config_version`, schema identity,
and digest bytes. Unknown and zero schema versions fail without mutation.

Version 1 is SHA-256 over the canonical Soroban XDR encoding of this tuple, in
exact field order:

```text
(
  1,
  "earnproof_schema_policy",
  schema_version,
  config_version,
  approved,
  activated_at,
  deprecated_at,
  proof_types,
  max_validity_seconds,
  payload_limit,
  rate_max_registrations,
  rate_window_ledgers
)
```

Proof-type order is preserved because it is governed policy data. Activation
and deprecation timestamps come only from the ledger environment. Approval,
deprecation, schema policy, payload policy, and rate policy mutations advance
`config_version`, so every accepted policy change yields a new commitment even
when governance replays the same value at the same ledger timestamp. Failed or
unauthorized mutations leave both the version and digest unchanged.

Lifecycle and rate state are versioned sidecars. Existing schema approval and
policy entries keep their prior storage encoding. A legacy schema with no
lifecycle sidecar reports zero timestamps; a schema with no explicit rate
policy uses the shared default (`u32::MAX` registrations per 1,000 ledgers).
