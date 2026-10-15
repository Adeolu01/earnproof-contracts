# Schema Deprecation Replacement Pointers

`deprecate_schema_to` atomically marks an approved schema unavailable for new
proofs and stores an immutable version-1 pointer to another currently approved
schema. The pointer includes an exact effective ledger and one public
compatibility classification: `BackwardCompatible`, `MigrationRequired`, or
`Breaking`.

The source and successor must both exist and be approved before the call. A
self-pointer, unknown successor, deprecated successor, replay, cycle, or chain
beyond the documented 32-link bound fails before any storage, history, version,
or event change. When threshold governance is enabled, callers must use
`CriticalAction::SchemaReplacement`; the direct admin entry point is disabled.

`get_schema_replacement` returns the direct pointer.
`get_schema_replacement_chain` returns the original version followed by at most
32 successor links. Replacement pointers do not rewrite schema identifiers in
existing proof records, so historical proofs retain their original identity.
