//! Host Beget inside Beget inside Beget.
//!
//! cargo run --release --example recursive_hosting

fn main() {
    let inner = beget::begetception::nest(3);
    println!("depth = {}, hops to bare metal = {}", inner.depth, inner.hops_to_metal());
}
