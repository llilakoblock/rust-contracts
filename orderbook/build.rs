use order_book_io::OrderBookMetadata;

fn main() {
    gear_wasm_builder::build_with_metadata::<OrderBookMetadata>();
}