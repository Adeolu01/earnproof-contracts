use earnproof_shared::{
    IssuerError, ISSUER_ASSURANCE_COMMITMENT_VERSION, MAX_ISSUER_ASSURANCE_HISTORY_PAGE,
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
fn assurance_changes_are_versioned_ordered_and_immutable() {
    let (env, client, _admin, issuer_id) = setup();
    let first =
        client.set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &2);
    assert_eq!(first.effective_ledger, 100);
    assert_eq!(first.sequence, 1);

    env.ledger().with_mut(|ledger| ledger.sequence_number = 101);
    let second =
        client.set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &3);
    assert_eq!(second.effective_ledger, 101);
    assert_eq!(second.sequence, 2);
    assert_eq!(
        client
            .get_issuer_assurance_at_sequence(&issuer_id, &1)
            .unwrap(),
        first
    );
    assert_eq!(
        client.get_issuer_assurance_level(&issuer_id).unwrap(),
        second
    );
    let history = client.get_issuer_assurance_history(&issuer_id, &0, &2);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0).unwrap().assurance_level, 2);
    assert_eq!(history.get(1).unwrap().assurance_level, 3);
}

#[test]
fn invalid_and_replayed_changes_do_not_advance_history() {
    let (_env, client, _admin, issuer_id) = setup();
    client.set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &1);
    assert_eq!(
        client
            .try_set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &1,),
        Err(Ok(IssuerError::InvalidTransition))
    );
    assert_eq!(
        client.try_set_issuer_assurance_level(&issuer_id, &2, &1),
        Err(Ok(IssuerError::InvalidAssuranceLevel))
    );
    assert_eq!(
        client
            .try_set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &0,),
        Err(Ok(IssuerError::InvalidAssuranceLevel))
    );
    assert_eq!(
        client
            .get_issuer_assurance_history(&issuer_id, &0, &MAX_ISSUER_ASSURANCE_HISTORY_PAGE,)
            .len(),
        1
    );
    assert_eq!(
        client.try_get_issuer_assurance_history(
            &issuer_id,
            &0,
            &(MAX_ISSUER_ASSURANCE_HISTORY_PAGE + 1),
        ),
        Err(Ok(IssuerError::BatchTooLarge))
    );
}

#[test]
fn assurance_history_rolls_over_without_reordering_retained_bindings() {
    let (_env, client, _admin, issuer_id) = setup();
    for sequence in 1..=18_u32 {
        client.set_issuer_assurance_level(
            &issuer_id,
            &ISSUER_ASSURANCE_COMMITMENT_VERSION,
            &sequence,
        );
    }

    assert!(client
        .get_issuer_assurance_at_sequence(&issuer_id, &2)
        .is_none());
    let history =
        client.get_issuer_assurance_history(&issuer_id, &0, &MAX_ISSUER_ASSURANCE_HISTORY_PAGE);
    assert_eq!(history.len(), MAX_ISSUER_ASSURANCE_HISTORY_PAGE);
    assert_eq!(history.get(0).unwrap().sequence, 3);
    assert_eq!(history.get(15).unwrap().sequence, 18);
}

#[test]
fn non_admin_cannot_change_assurance() {
    let (env, client, _admin, issuer_id) = setup();
    let outsider = Address::generate(&env);
    env.mock_auths(&[MockAuth {
        address: &outsider,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "set_issuer_assurance_level",
            args: (issuer_id.clone(), ISSUER_ASSURANCE_COMMITMENT_VERSION, 4u32).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(client
        .try_set_issuer_assurance_level(&issuer_id, &ISSUER_ASSURANCE_COMMITMENT_VERSION, &4,)
        .is_err());
    assert!(client.get_issuer_assurance_level(&issuer_id).is_none());
}
