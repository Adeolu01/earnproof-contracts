use earnproof_shared::{
    ContractError, CriticalAction, CriticalActionPolicy, SchemaActivationState,
    MAX_SCHEMA_ACTIVATION_NOTICE_SECONDS,
};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal,
};

fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);
    let contract_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

#[test]
fn activation_is_pending_until_the_exact_committed_boundary() {
    let (env, client, _admin) = setup();
    client.set_min_schema_activation_notice(&100);
    assert_eq!(
        client.try_approve_schema_version(&1),
        Err(Ok(ContractError::InvalidTimingConfig))
    );
    assert_eq!(
        client.try_approve_schema_at(&1, &1_099),
        Err(Ok(ContractError::InvalidTimingConfig))
    );
    assert_eq!(
        client.get_schema_activation_state(&1),
        SchemaActivationState::Unknown
    );

    client.approve_schema_at(&1, &1_100);
    let notice = client.get_schema_activation_notice(&1).unwrap();
    assert_eq!(notice.proposed_at, 1_000);
    assert_eq!(notice.activates_at, 1_100);
    assert_eq!(notice.minimum_notice_seconds, 100);
    assert_eq!(
        client.get_schema_activation_state(&1),
        SchemaActivationState::Pending
    );
    assert!(!client.is_schema_version_approved(&1));

    env.ledger().set_timestamp(1_099);
    assert!(!client.is_schema_version_approved(&1));
    env.ledger().set_timestamp(1_100);
    assert!(client.is_schema_version_approved(&1));
    assert_eq!(
        client.get_schema_activation_state(&1),
        SchemaActivationState::Active
    );
}

#[test]
fn shortening_policy_does_not_accelerate_pending_schema() {
    let (env, client, _admin) = setup();
    client.set_min_schema_activation_notice(&100);
    client.approve_schema_at(&1, &1_100);
    client.set_min_schema_activation_notice(&0);

    env.ledger().set_timestamp(1_001);
    assert!(!client.is_schema_version_approved(&1));
    assert_eq!(
        client
            .get_schema_activation_notice(&1)
            .unwrap()
            .activates_at,
        1_100
    );
    client.approve_schema_version(&2);
    assert!(client.is_schema_version_approved(&2));
}

#[test]
fn stale_threshold_proposal_and_unauthorized_policy_change_are_atomic() {
    let (env, client, admin) = setup();
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_min_schema_activation_notice",
            args: (100u64,).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client.try_set_min_schema_activation_notice(&100).is_err());
    assert_eq!(client.get_min_schema_activation_notice(), 0);

    env.mock_all_auths();
    assert_eq!(
        client.try_set_min_schema_activation_notice(&(MAX_SCHEMA_ACTIVATION_NOTICE_SECONDS + 1)),
        Err(Ok(ContractError::InvalidTimingConfig))
    );
    client.set_min_schema_activation_notice(&100);
    client.set_critical_action_policy(&CriticalActionPolicy {
        enabled: true,
        threshold: 1,
        signers: vec![&env, admin.clone()],
    });
    let proposal = client.propose_critical_action(&CriticalAction::SchemaActivation(1, 1_100));
    client.approve_critical_action(&proposal, &admin);
    env.ledger().set_timestamp(1_050);
    assert_eq!(
        client.try_execute_critical_action(&proposal),
        Err(Ok(ContractError::InvalidTimingConfig))
    );
    assert_eq!(
        client.get_schema_activation_state(&1),
        SchemaActivationState::Unknown
    );
    assert!(client.get_critical_action_proposal(&proposal).is_some());
}
