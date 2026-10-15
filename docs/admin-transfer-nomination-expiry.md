# Administrator transfer nomination expiry

All three contracts retain the existing two-step administrator transfer while
bounding every new nomination to `ADMIN_TRANSFER_EXPIRY_LEDGERS` ledgers. A
nomination is valid in the half-open interval
`[nominated_at_ledger, expires_at_ledger)`. Acceptance at the exact expiry
ledger returns `AdminNominationExpired` and leaves the current administrator
unchanged.

`get_admin_transfer_status` returns `Absent`, `Pending`, or `Expired`, the
nominated address, and both boundary ledgers. `cleanup_expired_admin_transfer`
is permissionless only after expiry. It deletes the nomination and its timing
sidecars, emits `AdminTransferExpired`, and cannot change administrator state.
Repeating cleanup returns `NotFound`.

The original `PendingAdmin` value remains an `Address`; the timing information
uses additive instance-storage sidecars. This preserves decoding compatibility.
A nomination created before the sidecars existed is reported as `Expired` with
zero boundary ledgers and must be cleaned or replaced by the current admin.

Only commitments and public ledger metadata are stored. No credentials,
private keys, incident payloads, or wallet material are accepted.
