use scrypto::prelude::*;

pub fn get_utilisation(deposit_balance: Decimal, borrow_balance: Decimal) -> Decimal {
    /*let borrow_precise_decimal: PreciseDecimal = borrow_balance.into();
    let deposit_precise_decimal: PreciseDecimal = deposit_balance.into();

    let utilisation_precise_decimal = borrow_precise_decimal / deposit_precise_decimal;
    let utilisation = Decimal::from(utilisation_precise_decimal.to_string()); */
    if deposit_balance.is_zero() {
        // Handle the case where deposit_balance is zero
        return Decimal::ZERO;
    }
    let deposit_balance = PreciseDecimal::try_from(deposit_balance).ok().unwrap();
    let borrow_balance = PreciseDecimal::try_from(borrow_balance).ok().unwrap();
    let utilisation = borrow_balance / deposit_balance;
    PreciseDecimal::try_from(utilisation)
        .ok()
        .and_then(|e| e.try_into().ok())
        .unwrap()
}

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

pub fn calculate_borrow_apr(borrow_rate: Decimal, updated_at: u64) -> Decimal {
    let borrow_apr =
        borrow_rate * (Runtime::current_epoch().number() - updated_at) / (365 * 24 * 60 / 5);
    borrow_apr
}

pub fn calculate_interests(
    total_borrow: Decimal,
    borrow_apr: Decimal,
    reserve_factor: Decimal,
) -> (Decimal, Decimal, Decimal) {
    let borrow_interest = total_borrow * borrow_apr;
    let reserve_interest = borrow_interest * reserve_factor;
    let deposit_interest = borrow_interest - reserve_interest;
    (borrow_interest, reserve_interest, deposit_interest)
}

pub fn calculate_token_price(total: Decimal, total_s: Decimal) -> Decimal {
    let mut token_price = Decimal::one();
    if total_s != Decimal::zero() {
        token_price = total / total_s;
    }
    token_price
}

pub fn calculate_s_interest(amount: Decimal, total: Decimal, total_s: Decimal) -> Decimal {
    let s_interest;
    if total == Decimal::zero() {
        s_interest = amount;
    } else {
        let s_price = total / total_s;
        s_interest = amount / s_price;
    }
    s_interest
}
