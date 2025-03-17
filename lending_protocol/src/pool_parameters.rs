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
    //Loan to value ratio
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
    //Platform reserve balance
    pub reserve_balance: Decimal,
    pub sd_balance: Decimal,
    pub sb_balance: Decimal,
    pub balances_updated_at: u64,
}
impl PoolParameters {
    pub fn _get_pool_parameters(&mut self) -> (Decimal, Decimal, Decimal, Decimal) {
        (
            self.max_liquidation_percent,
            self.liquidation_bonus,
            self.liquidation_reserve_factor,
            self.deposit_limit,
        )
    }

    pub fn update_pool_parameters(
        &mut self,
        liquidation_reserve_factor: Decimal,
        liquidation_bonus: Decimal,
        max_liquidation_percent: Decimal,
        max_borrow_percent: Decimal,
        min_collateral_ratio: Decimal,
        pool_reserve: Decimal,
        pool_deposit_limit: Decimal,
    ) {
        self.liquidation_reserve_factor = liquidation_reserve_factor;
        self.liquidation_bonus = liquidation_bonus;
        self.max_liquidation_percent = max_liquidation_percent;
        self.max_borrow_percent = max_borrow_percent;
        self.min_collateral_ratio = min_collateral_ratio;
        self.pool_reserve = pool_reserve;
        self.deposit_limit = pool_deposit_limit;
    }

    pub fn update_pool_settings(
        &mut self,
        base: Decimal,
        base_multiplier: Decimal,
        multiplier: Decimal,
        kink: Decimal,
        reserve_factor: Decimal,
        ltv_ratio: Decimal,
    ) {
        self.base = base;
        self.base_multiplier = base_multiplier;
        self.multiplier = multiplier;
        self.kink = kink;
        self.reserve_factor = reserve_factor;
        self.ltv_ratio = ltv_ratio;
    }

    pub fn update_balances(
        &mut self,
        deposit_balance: Decimal,
        sd_balance: Decimal,
        borrow_balance: Decimal,
        sb_balance: Decimal,
        reserve_balance: Decimal,
    ) {
        self.deposit_balance = deposit_balance;
        self.sd_balance = sd_balance;
        self.borrow_balance = borrow_balance;
        self.sb_balance = sb_balance;
        self.reserve_balance = reserve_balance;
        self.balances_updated_at = Runtime::current_epoch().number();
    }

    pub fn lock_pool(
        &mut self,
        deposit_locked: bool,
        borrow_locked: bool,
        withdraw_locked: bool,
        repay_locked: bool,
    ) {
        self.deposit_locked = deposit_locked;
        self.borrow_locked = borrow_locked;
        self.withdraw_locked = withdraw_locked;
        self.repay_locked = repay_locked;
    }
}
