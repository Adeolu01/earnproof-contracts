# Proof dispute filing windows

Protocol config stores one governed `DisputeFilingPolicy`. Version 1 selects a
lifecycle anchor (`Created`, `Activated`, `Expired`, or `Revoked`) and two
unsigned second offsets. Proof registry resolves the timestamps as follows:

| Anchor | Timestamp |
| --- | --- |
| `Created` | `ProofRecord.created_at` |
| `Activated` | `activates_at`, or `created_at` when activation is immediate |
| `Expired` | `expires_at` |
| `Revoked` | `revoked_at`; unavailable until the terminal revoked state exists |

The filing interval is half-open:

```text
anchor + opens_after_seconds <= ledger timestamp
ledger timestamp < anchor + closes_after_seconds
```

Both additions saturate at `u64::MAX`. Governance must use policy version 1
and a closing offset strictly greater than the opening offset. Failed updates
leave the policy and config version unchanged. A new or pre-policy deployment
uses the compatibility policy `Created, 0, u64::MAX`, preserving the prior
unbounded behavior until governance explicitly changes it.

Clients should call `get_dispute_filing_eligibility` immediately before
`open_dispute`. The query reports the policy version, resolved boundaries,
evaluation timestamp, and one of `Eligible`, `NotYetOpen`, `Closed`, or
`AnchorUnavailable`. `open_dispute` re-evaluates the policy atomically and
returns `ProofError::DisputeFilingWindowClosed` for every ineligible result.
An already-open dispute remains authoritative and returns
`ProofError::DisputeAlreadyOpen`; a terminal prior dispute cannot be replayed
after the current filing window closes.
