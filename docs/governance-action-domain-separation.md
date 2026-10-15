# Governance action commitment domain

Protocol-config interface `1.2.0` introduces governance action commitment
encoding version `1`. The proposal identifier is SHA-256 over the Soroban XDR
encoding of this ordered tuple:

1. the fixed `gov_action` domain tag;
2. encoding version `1`;
3. the current ledger network ID;
4. the protocol-config contract address;
5. the critical action category;
6. the complete typed critical action payload; and
7. the monotonic proposal nonce.

The network and contract fields prevent a proposal identifier from being
reused across deployments. Including both the category and typed payload keeps
approval, cancellation, and execution bound to exactly the action proposed.
Soroban XDR provides deterministic framing; clients can call
`derive_critical_action_id` to obtain the canonical identifier.

Every new proposal stores a `GovernanceActionDomain` sidecar. Approval,
cancellation, and execution recompute and validate the identifier before any
state mutation. A mismatch fails atomically with `InvalidState`. Successful
cancellation or execution removes both the proposal and its sidecar, so replay
returns `ApprovalProposalNotFound`.

## Compatibility

The proposal record itself is unchanged. Pending proposals written by an older
contract therefore remain decodable and retain their original embedded-policy
behavior. `get_critical_action_domain` returns `None` for those legacy records;
all proposals created by interface `1.2.0` or later return the explicit V1
domain metadata.
