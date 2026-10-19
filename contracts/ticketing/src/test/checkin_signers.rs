//! Delegated check-in signers (issue #89).

use super::*;

#[test]
fn listed_signer_can_check_in_and_organizer_still_can() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let gate = Address::generate(&env);
    let t1 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    let t2 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);

    client.add_checkin_signer(&organizer, &1, &gate);
    assert!(client.is_checkin_signer(&1, &gate));

    client.check_in(&gate, &t1);
    assert_eq!(client.get_ticket(&t1).status, TicketStatus::Used);
    client.check_in(&organizer, &t2);
    assert_eq!(client.get_ticket(&t2).status, TicketStatus::Used);
}

#[test]
fn non_listed_address_is_rejected() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let stranger = Address::generate(&env);
    let t = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);

    assert_eq!(
        client.try_check_in(&stranger, &t),
        Err(Ok(Error::NotOrganizer))
    );
    let mut ids = Vec::new(&env);
    ids.push_back(t);
    assert_eq!(
        client.try_check_in_batch(&stranger, &ids),
        Err(Ok(Error::NotOrganizer))
    );
    assert_eq!(client.get_ticket(&t).status, TicketStatus::Valid);
}

#[test]
fn removed_signer_and_other_events_signer_are_rejected() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    make_event(&env, &client, &organizer, 2);
    let owner = Address::generate(&env);
    let gate = Address::generate(&env);
    let t1 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    let t2 = issue_sample_ticket(&env, &client, &organizer, 2, &owner, 1_000);

    client.add_checkin_signer(&organizer, &1, &gate);
    // A signer is scoped to one event.
    assert_eq!(
        client.try_check_in(&gate, &t2),
        Err(Ok(Error::NotOrganizer))
    );

    client.remove_checkin_signer(&organizer, &1, &gate);
    assert!(!client.is_checkin_signer(&1, &gate));
    assert_eq!(
        client.try_check_in(&gate, &t1),
        Err(Ok(Error::NotOrganizer))
    );
}

#[test]
fn signer_can_batch_check_in_and_use_zones() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let owner = Address::generate(&env);
    let gate = Address::generate(&env);
    let t1 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    let t2 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    let t3 = issue_sample_ticket(&env, &client, &organizer, 1, &owner, 1_000);
    client.add_checkin_signer(&organizer, &1, &gate);

    let mut ids = Vec::new(&env);
    ids.push_back(t1);
    ids.push_back(t2);
    client.check_in_batch(&gate, &ids);
    client.check_in_at_zone(&gate, &t3, &String::from_str(&env, "north-gate"));
    assert_eq!(client.get_ticket(&t3).status, TicketStatus::Used);
}

#[test]
fn only_organizer_can_manage_signers() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let stranger = Address::generate(&env);
    let gate = Address::generate(&env);
    assert_eq!(
        client.try_add_checkin_signer(&stranger, &1, &gate),
        Err(Ok(Error::NotOrganizer))
    );
    client.add_checkin_signer(&organizer, &1, &gate);
    assert_eq!(
        client.try_remove_checkin_signer(&stranger, &1, &gate),
        Err(Ok(Error::NotOrganizer))
    );
    assert!(client.is_checkin_signer(&1, &gate));
}
