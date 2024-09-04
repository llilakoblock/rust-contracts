#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use gstd::{msg, prelude::*};
use fixed::types::I16F16;
use gstd::ptr::addr_of_mut;
use order_book_io::{*};
use sha2::Digest;
use gstd::fmt::{*};

static mut ORDER_BOOK: Vec<Order> = Vec::new();

#[no_mangle]
extern "C" fn handle() {
    let action: Action = msg::load().expect("Failed to load Action message");
    
    let subject: [u8; 32] = array::from_fn(|i| i as u8 + 1);
    let (seed, _block_number) = gstd::exec::random(subject).expect("Error in random");

    match action {
        Action::AddOrder(mut order) => {

        order.user = msg::source();
        order.id = encode_hex(&sha2_256(&seed)); 
        order.valid_until = gstd::exec::block_timestamp() + 86400000; //cuurent blocktime + 24 hours
        order.is_locked = false;
        order.is_inactive = false;
        order.inactive_time_start = 0;

        unsafe {
            ORDER_BOOK.push(order.clone());
        }

        msg::reply(Event::OrderAdded(order.clone()), 0).expect("Failed add order");
        
        },
        Action::DeleteOrder(id) => {
            unsafe {
                ORDER_BOOK.retain(|order| order.id != id)
            }

            msg::reply(Event::OrderDeleted(id), 0).expect("Failed to reply with OrderDeleted event");
        },
        Action::ModifyOrder (updated_order ) => {
            unsafe {
                if let Some(order) = ORDER_BOOK.iter_mut().find(|order| order.id == updated_order.id) {
                    *order = updated_order;

                    msg::reply(Event::OrderModified(order.clone()), 0).expect("Failed to reply with OrderModified event");
                }
            }
        },
        Action::CheckOrders (test_bool) => {
            let caller_id = msg::source();

            unsafe {
                let user_orders: Vec<Order> = ORDER_BOOK.iter()
                    .filter(|order| order.user == caller_id)
                    .cloned()
                    .collect();

                for user_order in &user_orders {
                    let procceed_match = |match_data: (Order, MatchType)| {
                        
                        mark_order_as_matched(addr_of_mut!(ORDER_BOOK), user_order);
                        mark_order_as_matched(addr_of_mut!(ORDER_BOOK), &match_data.0);

                        let order_pair_alice = OrderPair{n1: match_data.0.clone(), n2: user_order.clone(), role: SwapRole::Alice, match_type: match_data.1.clone()};

                        let order_pair_bob = OrderPair{n1: match_data.0.clone(), n2: user_order.clone(),  role: SwapRole::Bob, match_type: match_data.1.clone()};

                        msg::send(user_order.user, Event::OrderMatched(order_pair_alice.clone()), 0).expect("Failed to send order to order creator");

                        msg::send(match_data.0.user, Event::OrderMatched(order_pair_bob.clone()), 0).expect("Failed to send matched order to matched order creator");
                    };
                        
                if let Some(matched_order) = find_exact_matching_order(user_order) {
                    procceed_match(matched_order);
                } else if let Some(matched_order) = find_exact_matching_order_slippage(user_order) {
                    procceed_match(matched_order);
                } else if let Some(matched_order) = find_and_match_partial_order(user_order) {
                    procceed_match(matched_order);
                }

                };
            };
        }
    }
}

#[no_mangle]
extern "C" fn state() {
    unsafe{
        let caller_id = msg::source();
        let user_orders: Vec<Order> = ORDER_BOOK.iter()
            .filter(|order| order.user != caller_id)
            .cloned()
            .collect();

        msg::reply(user_orders.clone(), 0).expect("Failed to reply with State Orders");
    }
}

/// Finds an exact match.
fn find_exact_matching_order(new_order: &Order) -> Option<(Order, MatchType)> {
    unsafe {
        for existing_order in &ORDER_BOOK {
            if existing_order.user != new_order.user &&
               !existing_order.is_locked && !existing_order.is_inactive &&
               existing_order.alpha_asset.name == new_order.beta_asset.name &&
               existing_order.beta_asset.name == new_order.alpha_asset.name &&
               existing_order.alpha_asset.nominal_amount == new_order.beta_asset.nominal_amount &&
               existing_order.beta_asset.nominal_amount == new_order.alpha_asset.nominal_amount
               {
                   return Some((existing_order.clone(), MatchType::Exact));
                }
        }
    }
    None
}

/// Finds a matching order with slippage.
fn find_exact_matching_order_slippage(new_order: &Order) -> Option<(Order, MatchType)> {
    unsafe {
        let new_order_max_acceptable_price = I16F16::lit(&new_order.alpha_asset_price) * (I16F16::lit(&"1") + I16F16::lit(&new_order.user_slippage) / 100);
        let new_order_min_acceptable_price = I16F16::lit(&new_order.alpha_asset_price) * (I16F16::lit(&"1") - I16F16::lit(&new_order.user_slippage) / 100);

        for existing_order in &ORDER_BOOK {

            let existing_order_max_acceptable_price = I16F16::lit(&existing_order.alpha_asset_price) * (I16F16::lit(&"1") + I16F16::lit(&existing_order.user_slippage) / 100);
            let existing_order_min_acceptable_price = I16F16::lit(&existing_order.alpha_asset_price) * (I16F16::lit(&"1") - I16F16::lit(&existing_order.user_slippage) / 100);

            if existing_order.user != new_order.user &&
               !existing_order.is_locked &&
               existing_order.alpha_asset.name == new_order.beta_asset.name &&
               existing_order.beta_asset.name == new_order.alpha_asset.name &&

               new_order_max_acceptable_price >= existing_order_min_acceptable_price &&
               new_order_min_acceptable_price <= existing_order_max_acceptable_price
               {
                   return Some((existing_order.clone(), MatchType::WithSlippage));
               }
        }
    }
    None
}

/// Finds and matches a partial order if the new order is smaller than an existing order.
fn find_and_match_partial_order(new_order: &Order) -> Option<(Order, MatchType)> {
    unsafe {
        for existing_order in ORDER_BOOK.iter_mut() {
            if existing_order.user != new_order.user &&
               !existing_order.is_locked &&
               existing_order.alpha_asset.name == new_order.beta_asset.name &&
               existing_order.beta_asset.name == new_order.alpha_asset.name &&

               (existing_order.alpha_asset.nominal_amount > new_order.beta_asset.nominal_amount &&
               existing_order.beta_asset.nominal_amount > new_order.alpha_asset.nominal_amount) ||

               (existing_order.alpha_asset.nominal_amount < new_order.beta_asset.nominal_amount &&
               existing_order.beta_asset.nominal_amount < new_order.alpha_asset.nominal_amount)
               {
                    return Some((existing_order.clone(), MatchType::Partial))
               }
            }
        }
    None
}

/// Marks an order as matched.
fn mark_order_as_matched(order_book: *mut Vec<Order>, order: &Order) {
    unsafe {
        if !order_book.is_null() {
            let order_book_ref: &mut Vec<Order> = &mut *order_book;
            for o in order_book_ref.iter_mut() {
                if o.id == order.id {
                    o.is_locked = true;
                    break;
                }
            }
        }
    }
}

fn sha2_256(data: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    output.copy_from_slice(sha2::Sha256::digest(data).as_slice());
    output
}

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        write!(&mut s, "{:02x}", b).unwrap();
    }
    s
}