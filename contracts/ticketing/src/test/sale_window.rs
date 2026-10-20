//! Primary sale window (issue #94).

use super::*;

fn buy(env: &Env, client: &TicketingContractClient, buyer: &Address) -> Result<u64, Error> {
    client
        .try_purchase_primary(
            buyer,
            &1,
            &String::from_str(env, "GA"),
            &String::from_str(env, "unassigned"),
        )
        .map(|r| r.unwrap())
        .map_err(|e| e.unwrap())
}

fn setup_sale<'a>() -> (Env, TicketingContractClient<'a>, Address, Address) {
    let (env, client, _token, asset, _admin, organizer) = setup();
    make_event(&env, &client, &organizer, 1);
    client.set_tier_price(&organizer, &1, &String::from_str(&env, "GA"), &100);
    let buyer = Address::generate(&env);
    mint_tokens(&asset, &buyer, 10_000);
    (env, client, organizer, buyer)
}

#[test]
fn purchase_is_rejected_before_the_window_opens_and_after_it_closes() {
    let (env, client, organizer, buyer) = setup_sale();
    client.set_event_sale_window(&organizer, &1, &Some(1_000u64), &Some(2_000u64));
    assert_eq!(client.sale_window(&1), (Some(1_000), Some(2_000)));

    env.ledger().set_timestamp(999);
    assert_eq!(buy(&env, &client, &buyer), Err(Error::SaleNotStarted));

    env.ledger().set_timestamp(1_000);
    assert!(buy(&env, &client, &buyer).is_ok());

    env.ledger().set_timestamp(1_999);
    assert!(buy(&env, &client, &buyer).is_ok());

    env.ledger().set_timestamp(2_000);
    assert_eq!(buy(&env, &client, &buyer), Err(Error::SaleEnded));
}

#[test]
fn open_ended_windows_and_no_window() {
    let (env, client, organizer, buyer) = setup_sale();
    env.ledger().set_timestamp(5_000);
    assert!(buy(&env, &client, &buyer).is_ok());
    assert_eq!(client.sale_window(&1), (None, None));

    client.set_event_sale_window(&organizer, &1, &None, &Some(5_000u64));
    assert_eq!(buy(&env, &client, &buyer), Err(Error::SaleEnded));

    client.set_event_sale_window(&organizer, &1, &Some(6_000u64), &None);
    assert_eq!(buy(&env, &client, &buyer), Err(Error::SaleNotStarted));
    env.ledger().set_timestamp(1_000_000);
    assert!(buy(&env, &client, &buyer).is_ok());

    client.set_event_sale_window(&organizer, &1, &None, &None);
    assert_eq!(client.sale_window(&1), (None, None));
}

#[test]
fn invalid_or_unauthorized_window_is_rejected() {
    let (env, client, organizer, _buyer) = setup_sale();
    assert_eq!(
        client.try_set_event_sale_window(&organizer, &1, &Some(2_000u64), &Some(2_000u64)),
        Err(Ok(Error::InvalidSaleWindow))
    );
    let stranger = Address::generate(&env);
    assert_eq!(
        client.try_set_event_sale_window(&stranger, &1, &Some(1u64), &None),
        Err(Ok(Error::NotOrganizer))
    );
}
