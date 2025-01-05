use scrypto::prelude::*;

#[derive(ScryptoSbor, Clone)]
pub struct PoolParameters {
    pub min_collateral_ratio: Decimal,
    // The max percent of liquidity pool user can borrow
    pub max_borrow_percent: Decimal,
    // The max percent of debt user can liquidate
    pub max_liquidation_percent: Decimal,
    // Liquidation bonus
    pub liquidation_bonus: Decimal,

    pub ltv_ratio: Decimal,
    // Multipliers for the assets
    pub multiplier: Decimal,
    // Base multipliers for the assets
    pub base_multiplier: Decimal,
    // Bases for the assets
    pub base: Decimal,
    // Reserve factors for the assets
    pub reserve_factor: Decimal,
    // KINKs for the assets
    pub kink: Decimal,
    // Per asset/editable
    pub liquidation_reserve_factor: Decimal,
    // Minimum reasonable value to be liquidated
    pub min_liquidable_value: Decimal,
    //Deposit lock state (true if locked)
    pub deposit_locked: bool,
    //Borrow lock state (true if locked)
    pub borrow_locked: bool,
    //Withdraw lock state (true if locked)
    pub withdraw_locked: bool,
    //Repay lock state (true if locked)
    pub repay_locked: bool,
    //Percent of pool amount that can not be borrowed or withdrawn
    pub pool_reserve: Decimal,
    //Pool deposit limit in USD
    pub deposit_limit: Decimal,
    // Loan balances
    pub borrow_balance: Decimal,
    // Deposit balances
    pub deposit_balance: Decimal,
    pub reserve_balance: Decimal,
    pub sr_deposit_balance: Decimal,
    pub sr_borrow_balance: Decimal,
    pub balances_updated_at: u64,
}
impl PoolParameters {
    pub fn _get_pool_parameters(&mut self) -> (Decimal, Decimal, Decimal, Decimal, Decimal) {
        info!("get_pool_parameters initiated.");

        (
            self.min_liquidable_value,
            self.max_liquidation_percent,
            self.liquidation_bonus,
            self.liquidation_reserve_factor,
            self.deposit_limit,
        )
    }

    pub fn update_pool_parameters(
        &mut self,
        min_liquidable_value: Decimal,
        liquidation_reserve_factor: Decimal,
        liquidation_bonus: Decimal,
        max_liquidation_percent: Decimal,
        max_borrow_percent: Decimal,
        min_collateral_ratio: Decimal,
        pool_reserve: Decimal,
        pool_deposit_limit: Decimal,
    ) {
        self.min_liquidable_value = min_liquidable_value;
        self.liquidation_reserve_factor = liquidation_reserve_factor;
        self.liquidation_bonus = liquidation_bonus;
        self.max_liquidation_percent = max_liquidation_percent;
        self.max_borrow_percent = max_borrow_percent;
        self.min_collateral_ratio = min_collateral_ratio;
        self.pool_reserve = pool_reserve;
        self.deposit_limit = pool_deposit_limit;
    }

    pub fn update_balances(
        &mut self,
        deposit_balance: Decimal,
        sr_deposit_balance: Decimal,
        borrow_balance: Decimal,
        sr_borrow_balance: Decimal,
        reserve_balance: Decimal,
    ) {
        self.deposit_balance = deposit_balance;
        self.sr_deposit_balance = sr_deposit_balance;
        self.borrow_balance = borrow_balance;
        self.sr_borrow_balance = sr_borrow_balance;
        self.reserve_balance = reserve_balance;
        self.balances_updated_at = Runtime::current_epoch().number();
    }
}
