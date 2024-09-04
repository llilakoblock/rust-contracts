#![no_std]

extern crate alloc;

use gstd::msg;
use gstd::{prelude::*, ActorId};
use price_oracle_io::{*};

#[async_trait::async_trait]
pub trait OracleHandler {
    async fn request_value(&self);

    fn change_manager(&mut self, new_manager: ActorId);

    fn assert_only_owner(&self);
}

#[async_trait::async_trait]
impl OracleHandler for Oracle {
    async fn request_value(&self) {
        let value: u128 = msg::send_for_reply_as(self.manager, 0, 0, 0)
            .expect("Unable to send message to `manager`.")
            .await
            .expect("Unable to decode reply payload from `manager`.");

        msg::reply(Event::NewValue { value }, 0).expect("Unable to reply!.");
    }

    fn change_manager(&mut self, new_manager: ActorId) {
        self.assert_only_owner();

        self.manager = new_manager;

        msg::reply(Event::NewManager(new_manager), 0).expect("Unable to reply!.");
    }

    fn assert_only_owner(&self) {
        if msg::source() != self.owner {
            panic!("Only owner allowed to call this function!.");
        }
    }
}

#[no_mangle]
extern "C" fn handle() {
    let action: Action = msg::load().expect("Failed to load Actions");

    match action {
        Action::SetPrice { name, price, interval } => {
            if msg::source() == oracle.owner || msg::source() == oracle.manager {
                let prices = oracle.prices.entry(name.clone()).or_default();
                match interval.as_str() {
                    "minutely" => prices.minute = price,
                    "hourly" => prices.hourly = price,
                    "weekly" => prices.weekly = price,
                    _ => (),
                }

            set_prices(crypto, prices);
            let all_prices = get_all_prices();
            msg::reply(Event::AllPrices(all_prices), 0).expect("Failed to get Prices");
        },
        Action::GetPrices { crypto } => {
            let prices = get_prices(&crypto);
            msg::reply(Event::Prices(prices), 0).expect("Failed to get Crypto Price");
        },
    }
}

#[no_mangle]
extern "C" fn state() {
    let state = State::new();
    msg::reply(state, 0).expect("Failed get State");
}