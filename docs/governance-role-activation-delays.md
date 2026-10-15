# Governance role activation delays

The issuer registry stores an administrator-governed activation delay for each
grantable governance role. `set_role_activation_delay` accepts values from zero
through 120,960 ledgers. The reserved `Recovery` role cannot be configured or
granted by this interface.

When `grant_governance_role` succeeds, the assignment snapshots the later of
the caller's requested activation ledger and the current ledger plus the
configured role delay. Changing the delay later never changes an existing
assignment. This preserves the serialized assignment format and requires no
migration for legacy assignments.

`get_governance_assignment_state` reports `Pending`, `Active`, or `Expired`.
The activation boundary is inclusive and the expiration boundary is exclusive:
an assignment authorizes at `activation_ledger`, and no longer authorizes at
`expiration_ledger`. Pending and expired assignments never authorize role-based
operations.

Only the current administrator may change delays or grant roles. Grant proposal
identifiers retain their existing one-time execution protection, so a replay or
concurrent grant attempt fails without modifying an assignment.
