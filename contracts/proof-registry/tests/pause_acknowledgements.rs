use earnproof_shared::{ContractError, PauseAcknowledgementRole, PauseScope};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use proof_registry::{ProofRegistryContract, ProofRegistryContractClient};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env};

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn setup() -> (Env, ProofRegistryContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let config_id = env.register(ProtocolConfigContract, ());
    ProtocolConfigContractClient::new(&env, &config_id).initialize(&admin);
    let issuer_id = env.register(IssuerRegistryContract, ());
    IssuerRegistryContractClient::new(&env, &issuer_id).initialize(&admin);
    let proof_id = env.register(ProofRegistryContract, ());
    let client = ProofRegistryContractClient::new(&env, &proof_id);
    client.initialize(&admin, &issuer_id, &config_id);
    (env, client, admin)
}

#[test]
fn local_pause_acknowledgement_is_replay_safe_and_does_not_unpause() {
    let (env, client, _admin) = setup();
    let operator = Address::generate(&env);
    let first = hash(&env, 1);
    client.set_pause_acknowledger(&operator, &true);
    client.pause_scope(&PauseScope::Updates);

    let record = client.acknowledge_pause(&PauseScope::Updates, &first, &operator);
    assert_eq!(record.role, PauseAcknowledgementRole::IncidentOperator);
    assert_eq!(record.sequence, 0);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Updates, &first, &operator),
        Err(Ok(ContractError::InvalidInput))
    );
    let second = client.acknowledge_pause(&PauseScope::Updates, &hash(&env, 2), &operator);
    assert_eq!(second.sequence, 1);

    client.unpause_scope(&PauseScope::Updates);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Updates, &hash(&env, 3), &operator),
        Err(Ok(ContractError::InvalidState))
    );
}

#[test]
fn unauthorized_actor_cannot_acknowledge_or_create_history() {
    let (env, client, _admin) = setup();
    let outsider = Address::generate(&env);
    let commitment = hash(&env, 4);
    client.pause_scope(&PauseScope::Disputes);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Disputes, &commitment, &outsider),
        Err(Ok(ContractError::Unauthorized))
    );
    assert!(client.get_pause_acknowledgement(&commitment).is_none());
    assert!(client.get_pause_acknowledgements(&0, &20).is_empty());
}
