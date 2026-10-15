use earnproof_shared::{IssuerAssuranceLevelCommitment, ISSUER_ASSURANCE_COMMITMENT_VERSION};
use soroban_sdk::{xdr::ToXdr, Env};

#[test]
fn assurance_commitment_encoding_is_stable_and_field_sensitive() {
    let env = Env::default();
    let commitment = IssuerAssuranceLevelCommitment {
        commitment_version: ISSUER_ASSURANCE_COMMITMENT_VERSION,
        assurance_level: 3,
        effective_ledger: 42,
        sequence: 7,
    };
    let encoded = commitment.clone().to_xdr(&env);
    assert_eq!(encoded, commitment.clone().to_xdr(&env));

    let next_level = IssuerAssuranceLevelCommitment {
        assurance_level: 4,
        ..commitment.clone()
    };
    let next_sequence = IssuerAssuranceLevelCommitment {
        sequence: 8,
        ..commitment
    };
    assert_ne!(encoded, next_level.to_xdr(&env));
    assert_ne!(encoded, next_sequence.to_xdr(&env));
}
