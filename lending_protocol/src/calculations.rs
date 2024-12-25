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

pub fn calculate_reserve_interest(borrow_interest: Decimal, reserve_factor: Decimal) -> Decimal {
    let reserve_interest = borrow_interest * reserve_factor;
    reserve_interest
}

pub fn calculate_deposit_interest(borrow_interest: Decimal, reserve_interest: Decimal) -> Decimal {
    let deposit_interest = borrow_interest - reserve_interest;
    deposit_interest
}

pub fn calculate_sd_token_price(total_deposit: Decimal, total_sd: Decimal) -> Decimal {
    let sd_price = total_deposit / total_sd;
    sd_price
}

pub fn calculate_sd_reward(
    deposit_amount: Decimal,
    total_deposit: Decimal,
    total_sd: Decimal,
) -> Decimal {
    let sd_reward;
    if total_deposit == Decimal::zero() {
        sd_reward = deposit_amount;
    } else {
        let sd_price = total_deposit / total_sd;
        sd_reward = deposit_amount / sd_price;
    }
    sd_reward
}
