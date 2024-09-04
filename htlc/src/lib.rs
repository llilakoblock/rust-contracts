#![no_std]

use collections::HashMap;
use exchange_io::*;
use gstd::{exec, msg, prelude::*, ActorId};
use sha2::Digest;

#[derive(Debug, Clone, Default)]
pub struct Exchange {
    hashlock: [u8; 32],
    timelock: u64,
    value: u128,
    sender: ActorId,
    receiver: ActorId,
    state: ExchangeState,
}

impl Exchange {
    fn new(
        hashlock: [u8; 32],
        timelock: u64,
        value: u128,
        sender: ActorId,
        receiver: ActorId,
    ) -> Self {
        Exchange {
            hashlock,
            timelock,
            value,
            sender,
            receiver,
            state: ExchangeState::Funded,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct Exchanges {
    exchanges: HashMap<[u8; 32], Exchange>,
}

static mut STORE: Option<Exchanges> = None;

#[no_mangle]
extern "C" fn handle() {
    let action: ExchangeAction = msg::load().expect("Could not load ExchangeAction");

    let store = unsafe { STORE.get_or_insert_with(Exchanges::default) };

    let exchanges = &mut store.exchanges;

    let result = match action {
        ExchangeAction::Fund(hashlock, timelock, receiver) => {
            assert!(msg::value() > 0, "Amount must be greater than 0");
            assert!(
                timelock > exec::block_timestamp(),
                "Unlock time must be in the future"
            );
            assert!(!receiver.is_zero(), "Invalid receiver address");

            let mut data = [0u8; 96];
            data[0..32].copy_from_slice(msg::source().as_ref());
            data[32..64].copy_from_slice(receiver.as_ref());
            data[64..96].copy_from_slice(&hashlock);

            let exchange_id = sha2_256(&data);

            assert!(!exchanges.contains_key(&exchange_id), "Duplicate exchange",);

            let sender = msg::source();
            let value = msg::value();

            let exchange = Exchange::new(hashlock, timelock, value, sender, receiver);

            exchanges.insert(exchange_id, exchange);

            ExchangeEvent::Funded(exchange_id, receiver, value)
        }
        ExchangeAction::Redeem(exchange_id, secret) => {
            let exchange = exchanges.get_mut(&exchange_id).expect("Exchange not found");

            assert!(
                exchange.state == ExchangeState::Funded,
                "Invalid exchange state"
            );
            assert!(
                exchange.receiver == msg::source(),
                "Only receiver can redeem"
            );
            assert!(exchange.hashlock == sha2_256(&secret), "Invalid secret");

            msg::send(exchange.receiver, (), exchange.value).expect("Failed to send funds");

            exchange.state = ExchangeState::Redeemed;

            ExchangeEvent::Redeemed(exchange_id, secret)
        }
        ExchangeAction::Refund(exchange_id) => {
            let exchange = exchanges.get_mut(&exchange_id).expect("Exchange not found");

            assert!(
                exchange.state == ExchangeState::Funded,
                "Invalid exchange state"
            );
            assert!(
                exchange.timelock >= exec::block_timestamp(),
                "Exchange not expired"
            );
            assert!(exchange.sender == msg::source(), "Only sender can refund");

            msg::send(exchange.sender, (), exchange.value).expect("Failed to send funds");

            exchange.state = ExchangeState::Refunded;

            ExchangeEvent::Refunded(exchange_id)
        }
    };

    msg::reply(result, 0).expect("Failed to encode or reply with `Result<ExchangeEvent, Error>`");
}

#[no_mangle]
extern "C" fn state() {
    let exchange_id = msg::load_bytes().expect("Could not load exchange id bytes");

    let store = unsafe { STORE.get_or_insert_with(Exchanges::default) };

    let exchanges = &mut store.exchanges;

    let exchange = exchanges
        .get(exchange_id.as_slice())
        .expect("Exchange not found");

    msg::reply(exchange.state, 0).expect("Failed to share state");
}

fn sha2_256(data: &[u8]) -> [u8; 32] {
    let mut output = [0u8; 32];
    output.copy_from_slice(sha2::Sha256::digest(data).as_slice());
    output
}
