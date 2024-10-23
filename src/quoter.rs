use rust_decimal::{prelude::Zero, Decimal};

pub trait Quoter {
    fn quote<'a, B>(size: &Decimal, book: B) -> Decimal
    where
        B: Iterator<Item = (&'a Decimal, &'a Decimal)>,
    {
        let mut remaining = size.clone();
        let mut amount = Decimal::zero();

        for (price, quantity) in book {
            let min = quantity.min(&remaining).clone();
            amount += min * price;
            remaining -= min;
            if remaining.is_zero() {
                break;
            }
        }

        amount / size
    }

    /// Returns the price of a buy quantity for a market order.
    fn buy(&self, quantity: &Decimal) -> Decimal;
    /// Returns the price of a sell quantity for a market order.
    fn sell(&self, quantity: &Decimal) -> Decimal;
}
