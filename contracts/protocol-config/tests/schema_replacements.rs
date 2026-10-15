use earnproof_shared::{ContractError, CriticalAction, CriticalActionPolicy, SchemaCompatibility};
use protocol_config::{ProtocolConfigContract, ProtocolConfigContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal,
};

fn setup() -> (Env, ProtocolConfigContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.sequence_number = 100);
    let contract_id = env.register(ProtocolConfigContract, ());
    let client = ProtocolConfigContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    client.approve_schema_version(&1);
    client.approve_schema_version(&2);
    client.approve_schema_version(&3);
    (env, client, admin)
}

#[test]
fn replacement_commits_at_boundary_and_preserves_original_identity() {
    let (env, client, _admin) = setup();
    let pointer = client.deprecate_schema_to(&1, &2, &SchemaCompatibility::MigrationRequired);
    assert_eq!(pointer.pointer_version, 1);
    assert_eq!(pointer.deprecated_version, 1);
    assert_eq!(pointer.replacement_version, 2);
    assert_eq!(pointer.effective_ledger, 100);
    assert!(!client.is_schema_version_approved(&1));
    assert!(client.is_schema_version_approved(&2));
    assert_eq!(client.get_schema_replacement(&1), Some(pointer));
    let chain = client.get_schema_replacement_chain(&1);
    assert_eq!(chain.len(), 2);
    assert_eq!(chain.get(0), Some(1));
    assert_eq!(chain.get(1), Some(2));
    assert_eq!(client.get_schema_predecessor(&1), None);

    env.ledger().with_mut(|ledger| ledger.sequence_number = 101);
    assert_eq!(
        client.try_deprecate_schema_to(&1, &2, &SchemaCompatibility::MigrationRequired),
        Err(Ok(ContractError::InvalidState))
    );
}

#[test]
fn unknown_deprecated_and_cycle_targets_fail_atomically() {
    let (_env, client, _admin) = setup();
    let version_before = client.get_config_version();
    assert_eq!(
        client.try_deprecate_schema_to(&1, &99, &SchemaCompatibility::Breaking),
        Err(Ok(ContractError::InvalidState))
    );
    assert_eq!(
        client.try_deprecate_schema_to(&1, &1, &SchemaCompatibility::BackwardCompatible),
        Err(Ok(ContractError::InvalidInput))
    );
    assert_eq!(client.get_config_version(), version_before);
    assert!(client.get_schema_replacement(&1).is_none());

    client.deprecate_schema_to(&1, &2, &SchemaCompatibility::BackwardCompatible);
    assert_eq!(
        client.try_deprecate_schema_to(&2, &1, &SchemaCompatibility::BackwardCompatible),
        Err(Ok(ContractError::InvalidState))
    );
    assert!(client.is_schema_version_approved(&2));
    assert!(client.get_schema_replacement(&2).is_none());
}

#[test]
fn replacement_obeys_authorization_and_threshold_governance() {
    let (env, client, admin) = setup();
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "deprecate_schema_to",
            args: (1u32, 2u32, SchemaCompatibility::Breaking).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_deprecate_schema_to(&1, &2, &SchemaCompatibility::Breaking)
        .is_err());
    assert!(client.get_schema_replacement(&1).is_none());

    env.mock_all_auths();
    client.set_critical_action_policy(&CriticalActionPolicy {
        enabled: true,
        threshold: 1,
        signers: vec![&env, admin.clone()],
    });
    assert_eq!(
        client.try_deprecate_schema_to(&1, &2, &SchemaCompatibility::Breaking),
        Err(Ok(ContractError::ThresholdApprovalRequired))
    );
    let proposal = client.propose_critical_action(&CriticalAction::SchemaReplacement(
        1,
        2,
        SchemaCompatibility::Breaking,
    ));
    client.approve_critical_action(&proposal, &admin);
    client.execute_critical_action(&proposal);
    assert_eq!(
        client.get_schema_replacement(&1).unwrap().compatibility,
        SchemaCompatibility::Breaking
    );
}

#[test]
fn pending_activation_cannot_be_used_as_a_replacement() {
    let (env, client, _admin) = setup();
    env.ledger().set_timestamp(1_000);
    client.set_min_schema_activation_notice(&100);
    client.approve_schema_at(&4, &1_100);

    let version_before = client.get_config_version();
    assert_eq!(
        client.try_deprecate_schema_to(&1, &4, &SchemaCompatibility::BackwardCompatible),
        Err(Ok(ContractError::InvalidState))
    );
    assert_eq!(client.get_config_version(), version_before);
    assert!(client.is_schema_version_approved(&1));
    assert!(client.get_schema_replacement(&1).is_none());

    env.ledger().set_timestamp(1_100);
    client.deprecate_schema_to(&1, &4, &SchemaCompatibility::BackwardCompatible);
    assert_eq!(
        client
            .get_schema_replacement(&1)
            .unwrap()
            .replacement_version,
        4
    );
}
