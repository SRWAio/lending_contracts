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

    /// All users borrows.
    #[mutable]
    pub borrows: IndexMap<ResourceAddress, Decimal>,
}

impl UserData {
    pub fn get_deposit(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.deposits, resource_address)
    }

    pub fn get_borrow(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.borrows, resource_address)
    }

    fn get_value(map: &IndexMap<ResourceAddress, Decimal>, key: ResourceAddress) -> Decimal {
        map.get(&key).copied().unwrap_or(Decimal::ZERO)
    }

    pub fn update_deposit(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.deposits, res_address, value);
    }

    pub fn update_borrow(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.borrows, res_address, value);
    }

    pub fn on_deposit(&mut self, resource_address: ResourceAddress, sr_deposit_increase: Decimal) {
        let mut sr_deposit = self.get_deposit(resource_address);
        sr_deposit += sr_deposit_increase;
        self.update_deposit(resource_address, sr_deposit);
    }

    pub fn on_withdraw(&mut self, resource_address: ResourceAddress, sr_deposit_decrease: Decimal) {
        let mut sr_deposit = self.get_deposit(resource_address);
        sr_deposit -= sr_deposit_decrease;
        self.update_deposit(resource_address, sr_deposit);
    }

    pub fn on_borrow(&mut self, resource_address: ResourceAddress, sr_borrow_increase: Decimal) {
        let mut sr_borrow = self.get_borrow(resource_address);
        sr_borrow += sr_borrow_increase;
        self.update_borrow(resource_address, sr_borrow);
    }

    pub fn on_repay(
        &mut self,
        resource_address: ResourceAddress,
        sr_borrow_decrease: Decimal,
    ) -> Decimal {
        let mut sr_borrow = self.get_borrow(resource_address);
        sr_borrow -= sr_borrow_decrease;
        let mut to_return = Decimal::ZERO;
        if sr_borrow < Decimal::ZERO {
            to_return = Decimal::ZERO - sr_borrow;
            sr_borrow = Decimal::ZERO;
        }
        self.update_borrow(resource_address, sr_borrow);
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
            let sr_deposit_balance = self.get_deposit(asset_address.clone());
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
