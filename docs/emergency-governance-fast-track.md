# Emergency governance fast track

Protocol-config interface `1.2.0` provides a guardian fast track restricted to
two actions: global pause and scoped pause. The typed action set cannot express
unpause, contract upgrade, dependency replacement, schema mutation, or an
arbitrary call. Normal upgrade timelocks and critical-action governance remain
the default for every action outside this allow-list.

The emergency policy contains an enabled flag, guardian set, threshold, and
maximum proposal lifetime. An enabled fast-track threshold must be strictly
greater than the current normal critical-action threshold. Before normal
threshold governance is enabled, the admin may bootstrap the policy directly;
afterward, policy changes require a normal `EmergencyPolicyUpdate` critical
proposal.

An admin or current guardian may propose an emergency action. The proposal
stores only a non-zero 32-byte reason commitment, never raw incident material,
and its requested expiry must be after the current ledger and no later than the
policy maximum. Proposal identifiers commit to encoding version, network,
contract, typed action, reason commitment, expiry, and a monotonic nonce.

Guardians must belong to both the proposal snapshot and the current policy.
Removed guardians cannot add approvals and their earlier approvals stop
counting. Execution rechecks that the fast-track threshold remains stronger
than normal governance. At `expires_at` the window is closed. Successful
execution only sets pause state to `true`, consumes the proposal, records the
reason commitment in bounded config history, and is replay-safe.
