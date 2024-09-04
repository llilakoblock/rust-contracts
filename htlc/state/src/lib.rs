#![no_std]

use exchange_io::*;
use gstd::{prelude::*, ActorId};

#[gmeta::metawasm]
pub mod metafns {
    pub type State = HashMap<[u8; 32], Exchange>;

    pub fn get_exchange_state(state: State, exchange_id: [u8; 32]) -> ExchangeState {
        state
            .get(&exchange_id)
            .map(|exchange| exchange.state)
            .unwrap_or(ExchangeState::Invalid)
    }
}
