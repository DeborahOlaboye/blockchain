//! Multi-use tickets (passes) with use counters (issue #74).
//!
//! Each ticket allows `max_uses` entries, defaulting to one. A ticket stays
//! `Valid` until its uses are exhausted, then becomes `Used`.

use super::*;

/// Issues a ticket on an existing event and returns its id.
fn issued_ticket(env: &Env, client: &TicketingContractClient, organizer: &Address) -> u64 {
    issue_sample_ticket(env, client, organizer, 1, &Address::generate(env), 1_000)
}

#[test]
fn tickets_default_to_one_use_and_become_used_on_first_check_in() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);

    assert_eq!(client.ticket_max_uses(&ticket_id), 1);
    assert_eq!(client.ticket_used_count(&ticket_id), 0);

    client.check_in(&organizer, &ticket_id);

    assert_eq!(client.ticket_used_count(&ticket_id), 1);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);
}

#[test]
fn multi_use_ticket_stays_valid_until_its_uses_are_exhausted() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.set_ticket_max_uses(&organizer, &ticket_id, &3u32);

    client.check_in(&organizer, &ticket_id);
    client.check_in(&organizer, &ticket_id);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Valid);
    assert!(client.is_ticket_valid(&ticket_id));
    assert_eq!(client.ticket_used_count(&ticket_id), 2);

    client.check_in(&organizer, &ticket_id);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);
    assert_eq!(client.ticket_used_count(&ticket_id), 3);

    assert_eq!(
        client.try_check_in(&organizer, &ticket_id),
        Err(Ok(Error::AlreadyUsed))
    );
    assert_eq!(client.ticket_used_count(&ticket_id), 3);
}

#[test]
fn check_in_batch_consumes_one_use_per_ticket() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.set_ticket_max_uses(&organizer, &ticket_id, &2u32);

    let mut batch = Vec::new(&env);
    batch.push_back(ticket_id);
    client.check_in_batch(&organizer, &batch);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Valid);

    client.check_in_batch(&organizer, &batch);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);
    assert_eq!(client.ticket_used_count(&ticket_id), 2);
}

#[test]
fn max_uses_rejects_zero_and_values_below_the_uses_already_recorded() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.set_ticket_max_uses(&organizer, &ticket_id, &3u32);
    client.check_in(&organizer, &ticket_id);
    client.check_in(&organizer, &ticket_id);

    assert_eq!(
        client.try_set_ticket_max_uses(&organizer, &ticket_id, &0u32),
        Err(Ok(Error::InvalidMaxUses))
    );
    assert_eq!(
        client.try_set_ticket_max_uses(&organizer, &ticket_id, &1u32),
        Err(Ok(Error::InvalidMaxUses))
    );
    assert_eq!(client.ticket_max_uses(&ticket_id), 3);
}

#[test]
fn raising_max_uses_revives_an_exhausted_ticket() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.check_in(&organizer, &ticket_id);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);

    client.set_ticket_max_uses(&organizer, &ticket_id, &2u32);

    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Valid);
    client.check_in(&organizer, &ticket_id);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);
    assert_eq!(client.ticket_used_count(&ticket_id), 2);
}

#[test]
fn pass_out_returns_the_use_consumed_by_the_entry() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.set_event_max_reentries(&organizer, &1, &1u32);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.set_ticket_max_uses(&organizer, &ticket_id, &2u32);
    client.check_in(&organizer, &ticket_id);
    client.check_in(&organizer, &ticket_id);
    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Used);

    client.pass_out(&organizer, &ticket_id);

    assert_eq!(client.get_ticket(&ticket_id).status, TicketStatus::Valid);
    assert_eq!(client.ticket_used_count(&ticket_id), 1);
}

#[test]
fn burning_a_used_ticket_clears_its_use_counters() {
    let (env, client, _token, _token_asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let ticket_id = issued_ticket(&env, &client, &organizer);
    client.set_ticket_max_uses(&organizer, &ticket_id, &2u32);
    client.check_in(&organizer, &ticket_id);
    client.check_in(&organizer, &ticket_id);
    env.ledger()
        .set_timestamp(10_000 + USED_TICKET_RETENTION_SECONDS);

    client.burn_used_ticket(&organizer, &ticket_id);

    assert_eq!(client.ticket_max_uses(&ticket_id), 1);
    assert_eq!(client.ticket_used_count(&ticket_id), 0);
}
