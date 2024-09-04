#![no_std]

use gmeta::{InOut, Metadata};
use gstd::{prelude::*, ActorId};

pub struct ExchangeMetadata;

impl Metadata for ExchangeMetadata {
    type Init = ();
    type Handle = InOut<ExchangeAction, ExchangeEvent>;
    type Others = ();
    type Reply = ();
    type Signal = ();
    type State = InOut<[u8; 32], ExchangeState>;
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ExchangeAction {
    Fund([u8; 32], u64, ActorId),
    Redeem([u8; 32], [u8; 32]),
    Refund([u8; 32]),
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ExchangeEvent {
    Funded([u8; 32], ActorId, u128),
    Redeemed([u8; 32], [u8; 32]),
    Refunded([u8; 32]),
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ExchangeState {
    Invalid,
    Funded,
    Redeemed,
    Refunded,
    Expired,
}

#[derive(Debug, Copy, Clone, Encode, Decode, TypeInfo, PartialEq, Eq)]
#[codec(crate = gstd::codec)]
#[scale_info(crate = gstd::scale_info)]
pub enum ExchangeError {
    ZeroAmount,
    NotFutureTime,
    InvalidReceiverAddress,
    DuplicateExchange,
    NotReceiverRedeem,
    NotSenderRefund,
    NotExpiredForRefund,
}

impl Default for ExchangeState {
    fn default() -> Self {
        ExchangeState::Invalid
    }
}
