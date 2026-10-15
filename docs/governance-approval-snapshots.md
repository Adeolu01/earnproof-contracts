# Governance approval snapshots

Protocol-config interface `1.2.0` assigns a monotonic version to every critical
action policy and captures an immutable snapshot when a proposal is created.
The snapshot contains encoding version `1`, the policy version, the original
threshold, and a SHA-256 commitment to the policy's ordered signer vector.

The signer commitment is SHA-256 over Soroban XDR for the `gov_signers` tag,
snapshot encoding version, and complete signer vector. Signer ordering is part
of the governed policy encoding, so clients can reproduce the digest exactly.

Approval, cancellation, and execution validate the snapshot against the full
policy already embedded in the proposal. Later policy changes therefore cannot
lower the threshold or replace the eligible signer set for a pending proposal.
Failed validation returns `InvalidState` before state changes, and policy
version overflow also fails atomically.

## Compatibility

The original proposal encoding remains unchanged. A legacy proposal without a
snapshot continues to use its embedded policy, and
`get_critical_action_snapshot` returns `None` so indexers can identify that
state explicitly. Every new proposal has a versioned snapshot which is removed
together with the proposal after cancellation or successful execution.
