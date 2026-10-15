use earnproof_shared::{
    ContractError, GovernanceAssignmentState, GovernanceRole, IssuerError, IssuerStatus,
    MAX_GOVERNANCE_ROLE_ACTIVATION_DELAY_LEDGERS,
};
use issuer_registry::{IssuerRegistryContract, IssuerRegistryContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger as _, MockAuth, MockAuthInvoke},
    Address, BytesN, Env, IntoVal,
};

fn hash(env: &Env, value: u8) -> BytesN<32> {
    BytesN::from_array(env, &[value; 32])
}

fn setup() -> (
    Env,
    IssuerRegistryContractClient<'static>,
    Address,
    BytesN<32>,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.sequence_number = 100);
    let contract_id = env.register(IssuerRegistryContract, ());
    let client = IssuerRegistryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let issuer_id = hash(&env, 1);
    client.initialize(&admin);
    client.register_issuer(
        &issuer_id,
        &Address::generate(&env),
        &hash(&env, 2),
        &hash(&env, 3),
    );
    (env, client, admin, issuer_id)
}

#[test]
fn pending_role_authorizes_at_exact_activation_boundary() {
    let (env, client, _admin, issuer_id) = setup();
    let delegate = Address::generate(&env);
    client.set_role_activation_delay(&GovernanceRole::IssuerManagement, &5);
    client.grant_governance_role(
        &hash(&env, 10),
        &GovernanceRole::IssuerManagement,
        &delegate,
        &100,
        &Some(110),
    );
    let assignment = client
        .get_governance_assignment(&GovernanceRole::IssuerManagement, &delegate)
        .unwrap();
    assert_eq!(assignment.activation_ledger, 105);
    assert_eq!(
        client.get_governance_assignment_state(&GovernanceRole::IssuerManagement, &delegate),
        Some(GovernanceAssignmentState::Pending)
    );

    env.ledger().with_mut(|ledger| ledger.sequence_number = 104);
    assert_eq!(
        client.try_suspend_issuer_by_role(&hash(&env, 11), &issuer_id, &hash(&env, 12), &delegate,),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(client.get_issuer(&issuer_id).status, IssuerStatus::Active);

    env.ledger().with_mut(|ledger| ledger.sequence_number = 105);
    client.suspend_issuer_by_role(&hash(&env, 13), &issuer_id, &hash(&env, 14), &delegate);
    assert_eq!(
        client.get_issuer(&issuer_id).status,
        IssuerStatus::Suspended
    );
    assert_eq!(
        client.get_governance_assignment_state(&GovernanceRole::IssuerManagement, &delegate),
        Some(GovernanceAssignmentState::Active)
    );

    env.ledger().with_mut(|ledger| ledger.sequence_number = 110);
    assert_eq!(
        client.get_governance_assignment_state(&GovernanceRole::IssuerManagement, &delegate),
        Some(GovernanceAssignmentState::Expired)
    );
}

#[test]
fn shortening_delay_does_not_retroactively_activate_assignments() {
    let (env, client, _admin, _issuer_id) = setup();
    let first = Address::generate(&env);
    let second = Address::generate(&env);
    client.set_role_activation_delay(&GovernanceRole::IssuerManagement, &10);
    client.grant_governance_role(
        &hash(&env, 20),
        &GovernanceRole::IssuerManagement,
        &first,
        &100,
        &None,
    );
    client.set_role_activation_delay(&GovernanceRole::IssuerManagement, &0);

    assert_eq!(
        client
            .get_governance_assignment(&GovernanceRole::IssuerManagement, &first)
            .unwrap()
            .activation_ledger,
        110
    );
    assert_eq!(
        client.get_governance_assignment_state(&GovernanceRole::IssuerManagement, &first),
        Some(GovernanceAssignmentState::Pending)
    );
    client.grant_governance_role(
        &hash(&env, 21),
        &GovernanceRole::IssuerManagement,
        &second,
        &100,
        &None,
    );
    assert_eq!(
        client.get_governance_assignment_state(&GovernanceRole::IssuerManagement, &second),
        Some(GovernanceAssignmentState::Active)
    );
}

#[test]
fn delay_policy_is_admin_only_and_bounded() {
    let (env, client, _admin, _issuer_id) = setup();
    assert_eq!(
        client.try_set_role_activation_delay(
            &GovernanceRole::IssuerManagement,
            &(MAX_GOVERNANCE_ROLE_ACTIVATION_DELAY_LEDGERS + 1),
        ),
        Err(Ok(ContractError::InvalidTimingConfig))
    );

    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_role_activation_delay",
            args: (GovernanceRole::IssuerManagement, 5u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_set_role_activation_delay(&GovernanceRole::IssuerManagement, &5)
        .is_err());
    assert_eq!(
        client.get_role_activation_delay(&GovernanceRole::IssuerManagement),
        0
    );
}
