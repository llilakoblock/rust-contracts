#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct OrderBookMetadata;

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum NetworkIdentityType
{
    EMPTY,
    SOCKS5,
    #[default] ORDER_SERVICE,
    IPV4,
    IPV6,
    Vara
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct NetworkIdentity
{
    pub network_type: NetworkIdentityType,
    pub identity: String,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Participant
{
    pub wallet_uuid: String,
    pub network_identity: NetworkIdentity,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Ledger {
    pub name: String,
    pub network: String,
    pub chain_id: i64,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Asset {
    pub ledger: Ledger,
    pub name: String,
    pub nominal_amount: String,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct Order {
    pub id: String,
    pub user: ActorId,
    pub user_slippage: String,
    pub alpha_asset: Asset,
    pub beta_asset: Asset,
    pub alpha_asset_price: String,
    pub beta_asset_price: String,
    pub is_locked: bool,
    pub is_inactive: bool,
    pub inactive_time_start: u64,
    pub valid_until: u64,
    pub creator: Participant,
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum SwapRole
{
    #[default] EMPTY,
    Alice,
    Bob
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq, Default)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum MatchType
{
    #[default] None,
    Exact,
    Partial,
    WithSlippage,
    OneToMany
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub struct OrderPair
{
    pub n1: Order,
    pub n2: Order,
    pub role: SwapRole,
    pub match_type: MatchType
}

#[derive(Debug, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Action {
    AddOrder(Order),
    DeleteOrder(String),
    ModifyOrder (Order),
    CheckOrders(bool)
}

#[derive(Debug,  Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum Event {
    OrderAdded(Order),
    OrderDeleted(String),
    OrderModified(Order),
    OrderMatched(OrderPair),
}

impl Metadata for OrderBookMetadata {
    type Init = ();
    type Handle = InOut<Action, Event>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<(), Vec<Order>>;
}