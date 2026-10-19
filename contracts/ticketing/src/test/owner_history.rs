//! Bounded ownership history (issue #82).

use super::*;

#[test]
fn history_starts_empty_and_records_previous_owners_in_order() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let c = Address::generate(&env);
    let id = issue_sample_ticket(&env, &client, &organizer, 1, &a, 1_000);
    assert!(client.owner_history(&id).is_empty());

    client.transfer_ticket(&a, &id, &b);
    client.transfer_ticket(&b, &id, &c);

    let history = client.owner_history(&id);
    assert_eq!(history.len(), 2);
    assert_eq!(history.get(0), Some(a));
    assert_eq!(history.get(1), Some(b));
    assert_eq!(client.owner_of(&id), c);
}

#[test]
fn history_is_bounded_to_the_most_recent_owners() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let mut owners: Vec<Address> = Vec::new(&env);
    for _ in 0..(MAX_OWNER_HISTORY + 3) {
        owners.push_back(Address::generate(&env));
    }
    let id = issue_sample_ticket(
        &env,
        &client,
        &organizer,
        1,
        &owners.get_unchecked(0),
        1_000,
    );
    for i in 1..owners.len() {
        client.transfer_ticket(&owners.get_unchecked(i - 1), &id, &owners.get_unchecked(i));
    }

    let history = client.owner_history(&id);
    assert_eq!(history.len(), MAX_OWNER_HISTORY);
    // Oldest entries dropped: the last N previous owners remain, in order.
    // The previous owners are owners[0..len-1]; the last N of those start here.
    let first_kept = owners.len() - 1 - MAX_OWNER_HISTORY;
    for i in 0..MAX_OWNER_HISTORY {
        assert_eq!(
            history.get_unchecked(i),
            owners.get_unchecked(first_kept + i)
        );
    }
}

#[test]
fn history_is_per_ticket() {
    let (env, client, _token, _asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    let a = Address::generate(&env);
    let b = Address::generate(&env);
    let moved = issue_sample_ticket(&env, &client, &organizer, 1, &a, 1_000);
    let kept = issue_sample_ticket(&env, &client, &organizer, 1, &a, 1_000);
    client.transfer_ticket(&a, &moved, &b);

    assert_eq!(client.owner_history(&moved).len(), 1);
    assert!(client.owner_history(&kept).is_empty());
}
