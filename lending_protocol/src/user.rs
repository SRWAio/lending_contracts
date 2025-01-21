use scrypto::prelude::*;

use crate::pool_parameters::PoolParameters;

/// Data describing the CDP
#[derive(ScryptoSbor, NonFungibleData, Clone, Debug)]
pub struct UserData {
    /// Image to display when exploring Radix transactions
    #[mutable]
    pub image_url: String,

    /// Name of user badge
    #[mutable]
    pub name: String,

    /// Badge minted at
    pub minted_at: u64,

    /// Badge updated at
    #[mutable]
    pub updated_at: u64,

    /// All users deposits.
    #[mutable]
    pub deposits: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub deposit_position: IndexMap<ResourceAddress, Decimal>,
    /* #[mutable]
    pub deposit_epoch: IndexMap<ResourceAddress, i64>,*/
    /// All users borrows.
    #[mutable]
    pub borrows: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub borrow_position: IndexMap<ResourceAddress, Decimal>,
    /* #[mutable]
    pub borrow_epoch: IndexMap<ResourceAddress, i64>,*/
}

impl UserData {
    pub fn get_deposits(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.deposits, resource_address)
    }

    pub fn get_deposit_position(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.deposit_position, resource_address)
    }

    pub fn get_borrows(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.borrows, resource_address)
    }

    pub fn get_borrow_position(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.borrow_position, resource_address)
    }

    fn get_value(map: &IndexMap<ResourceAddress, Decimal>, key: ResourceAddress) -> Decimal {
        map.get(&key).copied().unwrap_or(Decimal::ZERO)
    }

    pub fn update_deposit(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.deposits, res_address, value);
    }

    pub fn update_deposit_position(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.deposit_position, res_address, value);
    }

    pub fn update_borrow(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.borrows, res_address, value);
    }

    pub fn update_borrow_position(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.borrow_position, res_address, value);
    }

    pub fn on_deposit(
        &mut self,
        resource_address: ResourceAddress,
        deposit_increase: Decimal,
        sr_deposit_increase: Decimal,
        sd_price: Decimal,
    ) {
        let mut deposit = self.get_deposits(resource_address);
        deposit = deposit * sd_price + deposit_increase;
        let mut sr_deposit = self.get_deposit_position(resource_address);
        sr_deposit += sr_deposit_increase;
        self.update_deposit(resource_address, deposit);
        self.update_deposit_position(resource_address, sr_deposit);
    }

    pub fn on_withdraw(
        &mut self,
        resource_address: ResourceAddress,
        deposit_decrease: Decimal,
        sr_deposit_decrease: Decimal,
        sd_price: Decimal,
    ) {
        let mut deposit = self.get_deposits(resource_address);
        deposit = deposit * sd_price - deposit_decrease;
        let mut sr_deposit = self.get_deposit_position(resource_address);
        sr_deposit -= sr_deposit_decrease;
        self.update_deposit(resource_address, deposit);
        self.update_deposit_position(resource_address, sr_deposit);
    }

    pub fn on_borrow(
        &mut self,
        resource_address: ResourceAddress,
        borrow_increase: Decimal,
        sr_borrow_increase: Decimal,
        sb_price: Decimal,
    ) {
        let mut borrow = self.get_borrows(resource_address);
        borrow = borrow * sb_price + borrow_increase;
        let mut sr_borrow = self.get_borrow_position(resource_address);
        sr_borrow += sr_borrow_increase;
        self.update_borrow(resource_address, borrow);
        self.update_borrow_position(resource_address, sr_borrow);
    }

    pub fn on_repay(
        &mut self,
        resource_address: ResourceAddress,
        borrow_decrease: Decimal,
        sr_borrow_decrease: Decimal,
        sb_price: Decimal,
    ) -> Decimal {
        let mut borrow = self.get_borrows(resource_address);
        borrow = borrow * sb_price - borrow_decrease;
        let mut sr_borrow = self.get_borrow_position(resource_address);
        sr_borrow -= sr_borrow_decrease;
        let mut to_return = Decimal::ZERO;
        if sr_borrow < Decimal::ZERO {
            to_return = Decimal::ZERO - sr_borrow;
            sr_borrow = Decimal::ZERO;
            borrow = Decimal::ZERO;
        }
        self.update_borrow(resource_address, borrow);
        self.update_borrow_position(resource_address, sr_borrow);
        to_return
    }

    pub fn calculate_total_collateral(
        &mut self,
        pool_parameters: &HashMap<ResourceAddress, PoolParameters>,
        prices: HashMap<ResourceAddress, Decimal>,
    ) -> Decimal {
        // Total collateral a user has across all their assets
        let mut user_collateral_sum: Decimal = 0.into();

        // Iterate over each asset and calculate the amount of collateral available from each
        for (asset_address, parameters) in pool_parameters {
            let sr_deposit_balance = self.get_deposit_position(asset_address.clone());
            if sr_deposit_balance != Decimal::ZERO {
                let cost_of_asset_in_terms_of_xrd = prices.get(asset_address).unwrap();

                let sd_price = parameters.deposit_balance / parameters.sr_deposit_balance;
                let ltv_ratio = &parameters.ltv_ratio;

                let asset_value_in_xrd =
                    sr_deposit_balance * sd_price * *cost_of_asset_in_terms_of_xrd;
                let asset_collateral = asset_value_in_xrd * *ltv_ratio;
                user_collateral_sum += asset_collateral;
            }
        }
        user_collateral_sum.into()
    }

    fn update_map(
        map: &mut IndexMap<ResourceAddress, Decimal>,
        key: ResourceAddress,
        units: Decimal,
    ) {
        map.insert(key, units);
    }
}
