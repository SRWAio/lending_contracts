use scrypto::prelude::*;

pub fn calculate_borrow_rate(
    multiplier: Decimal,
    base_multiplier: Decimal,
    base: Decimal,
    kink: Decimal,
    utilisation: Decimal,
) -> Decimal {
    if utilisation > Decimal::from(0) {
        if utilisation < kink {
            let borrow_rate = base + base_multiplier * utilisation;
            borrow_rate
        } else {
            let borrow_rate = base + base_multiplier * kink + multiplier * (utilisation - kink);
            borrow_rate
        }
    } else {
        Decimal::from(0)
    }
}
