use earnproof_shared::{
    ContractError, PauseAcknowledgementRole, PauseScope, MAX_PAUSE_ACKNOWLEDGEMENT_HISTORY_PAGE,
};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.sequence_number = 500);
    let contract_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn incident_operator_acknowledges_without_changing_pause_state() {
    let (env, client, _admin) = setup();
    let operator = Address::generate(&env);
    let commitment = hash(&env, 1);
    client.set_pause_acknowledger(&operator, &true);
    client.set_scoped_pause(&PauseScope::Disputes, &true);

    let record = client.acknowledge_pause(&PauseScope::Disputes, &commitment, &operator);
    assert_eq!(record.record_version, 1);
    assert_eq!(record.sequence, 0);
    assert_eq!(record.role, PauseAcknowledgementRole::IncidentOperator);
    assert_eq!(record.acknowledged_at_ledger, 500);
    assert!(client.is_scope_paused(&PauseScope::Disputes));
    assert_eq!(
        client.get_pause_acknowledgement(&commitment),
        Some(record.clone())
    );
    assert_eq!(
        client.get_pause_acknowledgements(&0, &1).get(0),
        Some(record)
    );
}

#[test]
fn acknowledgement_requires_active_pause_and_rejects_replay() {
    let (env, client, admin) = setup();
    let commitment = hash(&env, 2);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Global, &commitment, &admin),
        Err(Ok(ContractError::InvalidState))
    );
    client.pause();
    client.acknowledge_pause(&PauseScope::Global, &commitment, &admin);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Global, &commitment, &admin),
        Err(Ok(ContractError::InvalidInput))
    );
    assert_eq!(
        client.try_get_pause_acknowledgements(&0, &(MAX_PAUSE_ACKNOWLEDGEMENT_HISTORY_PAGE + 1),),
        Err(Ok(ContractError::BatchTooLarge))
    );
    assert!(client.is_paused());
}

#[test]
fn acknowledgement_roles_are_admin_governed() {
    let (env, client, _admin) = setup();
    let outsider = Address::generate(&env);
    client.set_scoped_pause(&PauseScope::Updates, &true);
    assert_eq!(
        client.try_acknowledge_pause(&PauseScope::Updates, &hash(&env, 3), &outsider),
        Err(Ok(ContractError::Unauthorized))
    );

    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_pause_acknowledger",
            args: (outsider.clone(), true).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_set_pause_acknowledger(&outsider, &true).is_err());
    assert!(!client.is_pause_acknowledger(&outsider));
}
