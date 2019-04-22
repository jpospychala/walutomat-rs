extern crate walutomat;

use std::env;

fn main() {
  let wt = walutomat::v2::API::new("https://api.walutomat.pl", &env::var("WT_KEY").unwrap());
  let balance = wt.account_balance();
  for b in balance.unwrap().result.unwrap() {
    println!("{} {} (blocked {})", b.currency, b.balance_total, b.balance_reserved);
  }
}
