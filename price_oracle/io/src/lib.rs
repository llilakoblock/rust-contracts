#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct OracleMetadata;

impl Metadata for OracleMetadata {
    type Init = ();
    type Handle = InOut<Action, Event>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<(), State>;
}

#[derive(Debug, Default, Clone, Encode, Decode, TypeInfo)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Oracle {
    pub owner: ActorId,
    pub manager: ActorId,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Action {
    SetPrice { name: String, price: String, interval: String },
    GetPrices { crypto: String },
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    PricesSet,
    Prices(Option<Prices>),
    SpecificPrice(Option<Prices>),
    AllPrices(BTreeMap<String, Prices>),
    NewManager(ActorId),
    NewValue { value: u128 },
    Unauthorized,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Prices {
    pub minute: String,
    pub hourly: String,
    pub weekly: String,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct CryptoPrices {
    pub prices: BTreeMap<String, Prices>,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct State {
    pub authorized_users: Vec<ActorId>,
    pub crypto_prices: BTreeMap<String, Prices>,
}

static mut ORACLE_STATE: State = State {
    authorized_users: Vec::new(),
    crypto_prices: BTreeMap::new(),
};

pub fn is_authorized(user: ActorId) -> bool {
    unsafe { ORACLE_STATE.authorized_users.contains(&user) }
}

pub fn set_prices(crypto: String, prices: Prices) {
    unsafe { ORACLE_STATE.crypto_prices.insert(crypto, prices); }
}

pub fn get_prices(crypto: &String) -> Option<Prices> {
    unsafe { ORACLE_STATE.crypto_prices.get(crypto).cloned() }
}

pub fn get_all_prices() -> BTreeMap<String, Prices> {
    unsafe { ORACLE_STATE.crypto_prices.clone() }
}

impl State {
    pub fn new() -> Self {
        unsafe { ORACLE_STATE.clone() }
    }
}
