# Minimum Schema Activation Notice

Protocol governance configures the notice required for future schema
activations with `set_min_schema_activation_notice`. The value is measured in
ledger timestamp seconds, defaults to zero for compatibility, and is bounded to
one year. A nonzero policy disables immediate legacy approvals: callers use
`approve_schema_at` or the threshold-governed `SchemaActivation` action.

At scheduling time the contract requires:

`activates_at >= current ledger timestamp + current minimum notice`

Checked arithmetic rejects timestamp overflow. The accepted record snapshots
the proposal timestamp, activation timestamp, and then-current policy. Later
policy changes never rewrite the record, so shortening notice cannot accelerate
a pending schema. The schema is pending strictly before `activates_at` and
active at the exact boundary. Deprecation remains authoritative at every time.

Legacy approved schemas without a notice sidecar remain active. Public queries
expose the immutable notice and `Unknown`, `Pending`, `Active`, or `Deprecated`
state. Failed, stale, replayed, and unauthorized calls write no schedule,
configuration history, or events.
