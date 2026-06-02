use std::collections::{BTreeMap, VecDeque};

#[derive(Debug, Clone, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u32,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct OrderBook {
    pub bids: BTreeMap<u32, VecDeque<Order>>,
    pub asks: BTreeMap<u32, VecDeque<Order>>,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    pub fn add_resting(&mut self, order: Order) {
        match order.side {
            Side::Buy => {
                self.bids
                    .entry(order.price) // look up this price in the bids map
                    .or_default() // get the queue there, or make an empty one
                    .push_back(order); // add the order to the back of that queue
            }
            Side::Sell => {
                self.asks
                    .entry(order.price) // look up this price in the bids map
                    .or_default() // get the queue there, or make an empty one
                    .push_back(order); // add the order to the back of that queue
            }
        }
    }

    pub fn match_order(&mut self, mut taker: Order) -> Vec<Fill> {
        let mut fills = Vec::new();

        match taker.side {
            // A BUY matches against the ASKS, cheapest first.
            Side::Buy => {
                while taker.quantity > 0 {
                    // Find the lowest ask price. If there are no asks, stop.
                    let best_price = match self.asks.keys().next() {
                        Some(&p) => p,
                        None => break,
                    };
                    // If the cheapest ask costs more than we're willing to pay, stop.
                    if best_price > taker.price {
                        break;
                    }

                    let level = self.asks.get_mut(&best_price).unwrap(); // the queue at that price
                    let maker = level.front_mut().unwrap(); // the oldest order there

                    // Trade as much as both sides allow.
                    let fill_qty = taker.quantity.min(maker.quantity);
                    fills.push(Fill {
                        price: best_price, // fill at the MAKER's price (§7.4)
                        quantity: fill_qty,
                        maker_id: maker.id,
                        taker_id: taker.id,
                    });
                    maker.quantity -= fill_qty;
                    taker.quantity -= fill_qty;

                    // If the maker is fully filled, remove it; if the level is now empty, drop it.
                    if maker.quantity == 0 {
                        level.pop_front();
                        if level.is_empty() {
                            self.asks.remove(&best_price);
                        }
                    }
                }
            }

            // A SELL matches against the BIDS, highest first.
            Side::Sell => {
                while taker.quantity > 0 {
                    // Find the lowest ask price. If there are no asks, stop.
                    let best_price = match self.bids.keys().next_back() {
                        Some(&p) => p,
                        None => break,
                    };
                    // If the cheapest ask costs more than we're willing to pay, stop.
                    if best_price < taker.price {
                        break;
                    }

                    let level = self.bids.get_mut(&best_price).unwrap(); // the queue at that price
                    let maker = level.front_mut().unwrap(); // the oldest order there

                    // Trade as much as both sides allow.
                    let fill_qty = taker.quantity.min(maker.quantity);
                    fills.push(Fill {
                        price: best_price, // fill at the MAKER's price (§7.4)
                        quantity: fill_qty,
                        maker_id: maker.id,
                        taker_id: taker.id,
                    });
                    maker.quantity -= fill_qty;
                    taker.quantity -= fill_qty;

                    // If the maker is fully filled, remove it; if the level is now empty, drop it.
                    if maker.quantity == 0 {
                        level.pop_front();
                        if level.is_empty() {
                            self.bids.remove(&best_price);
                        }
                    }
                }
            }
        }

        // Whatever didn't fill rests on the book (limit-order behavior).
        if taker.quantity > 0 {
            self.add_resting(taker);
        }

        fills
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fill {
    pub price: u32,
    pub quantity: u32,
    pub maker_id: u64, // the resting order that was sitting on the book
    pub taker_id: u64, // the incoming order that matched against it
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn buy_matches_cheapest_asks_first() {
        let mut book = OrderBook::new();

        // Two resting sell orders sitting on the book.
        book.add_resting(Order {
            id: 1,
            side: Side::Sell,
            price: 60,
            quantity: 5,
        });
        book.add_resting(Order {
            id: 2,
            side: Side::Sell,
            price: 61,
            quantity: 8,
        });

        // A buyer wants 10 contracts, willing to pay up to 62c.
        let fills = book.match_order(Order {
            id: 3,
            side: Side::Buy,
            price: 62,
            quantity: 10,
        });

        // We expect: 5 @ 60c (from order 1), then 5 @ 61c (from order 2).
        assert_eq!(
            fills,
            vec![
                Fill {
                    price: 60,
                    quantity: 5,
                    maker_id: 1,
                    taker_id: 3
                },
                Fill {
                    price: 61,
                    quantity: 5,
                    maker_id: 2,
                    taker_id: 3
                },
            ]
        );
    }

    #[test]
    fn sell() {
        let mut arrange = OrderBook::new();

        arrange.add_resting(Order {
            id: 1,
            side: Side::Buy,
            price: 40,
            quantity: 7,
        });
        arrange.add_resting(Order {
            id: 2,
            side: Side::Buy,
            price: 39,
            quantity: 15,
        });

        let act = arrange.match_order(Order {
            id: 3,
            side: Side::Sell,
            price: 38,
            quantity: 13,
        });

        assert_eq!(
            act,
            vec![
                Fill {
                    price: 40,
                    quantity: 7,
                    maker_id: 1,
                    taker_id: 3
                },
                Fill {
                    price: 39,
                    quantity: 6,
                    maker_id: 2,
                    taker_id: 3
                },
            ]
        );
    }
}
