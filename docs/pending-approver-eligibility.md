# Pending proposal approver eligibility

Protocol-config interface `1.2.0` revalidates approvers for pending critical
actions. An address may approve only when it belongs to the proposal's
original signer set and remains a member of the current enabled policy.

Every policy membership grant advances a per-address assignment version. A
proposal captures those versions in a sidecar, so removing and later re-adding
an address does not revive its authority over an older proposal. New policy
members also cannot approve proposals that predate their membership. Existing
approvals remain in the audit record but stop counting toward execution while
their signer is removed, expired, or reassigned. The original proposal
threshold remains authoritative, so policy changes cannot lower it.

`approval_eligibility` reports one deterministic state: eligible, missing or
expired proposal, already approved, absent from the original snapshot, current
policy disabled, removed from the current policy, or reassigned after the
snapshot. The expiry boundary is closed: at `expires_at`, eligibility is
`ProposalExpired`.

Issuer-registry interface `1.1.0` exposes `governance_role_state`, which
classifies an assignment as missing, pending, active, or expired using the same
closed expiration boundary. Pending and expired assignments never authorize
role operations.

Legacy proposals without an approver-generation sidecar retain current-policy
revalidation but cannot distinguish removal followed by regrant. All proposals
created by interface `1.2.0` include the sidecar.
