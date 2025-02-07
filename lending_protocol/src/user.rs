use core::panic;

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

    pub fn on_liquidate_repay(
        &mut self,
        amount: Decimal,
        resource_address: ResourceAddress,
        cost_of_asset_in_terms_of_xrd: Decimal,
        sb_price: Decimal,
    ) -> Decimal {
        let sr_borrow = self.get_borrow(resource_address);
        // Increase borrow balance by interests accrued
        let mut new_sr_borrow = sr_borrow * sb_price;
        let mut interest = new_sr_borrow - sr_borrow;
        interest *= cost_of_asset_in_terms_of_xrd;
        new_sr_borrow *= cost_of_asset_in_terms_of_xrd;
        // Repay the loan
        if new_sr_borrow < amount {
            panic!("Amount is greater than borrow balance");
        } else {
            new_sr_borrow -= amount;
            new_sr_borrow /= cost_of_asset_in_terms_of_xrd;
            self.update_borrow(resource_address.clone(), new_sr_borrow);
            interest
        }
    }

    pub fn on_liquidate(
        &mut self,
        mut amount: Decimal,
        repaid_asset_address: ResourceAddress,
        borrow_amount: Decimal,
        deposit_amount: Decimal,
        mut max_liquidation_percent: Decimal,
        liquidation_bonus: Decimal,
        liquidation_reserve_factor: Decimal,
        mut liquidated_user_deposit_balance: Decimal,
        deposit_asset_address: ResourceAddress,
        prices: HashMap<ResourceAddress, Decimal>,
        //asset_min_liquidate_value: Decimal,
        asset_borrow_amount: Decimal,
        borrowable_amount: Decimal,
        sb_price: Decimal,
    ) -> (Decimal, Decimal, Decimal) {
        // Calculate the max repayment amount that's going to be used to repay users debt
        let cost_of_deposit_asset_in_terms_of_xrd = prices.get(&deposit_asset_address).unwrap();
        let cost_of_repaid_asset_in_terms_of_xrd = prices.get(&repaid_asset_address).unwrap();
        let borrowable_amount_in_terms_of_xrd =
            borrowable_amount * *cost_of_deposit_asset_in_terms_of_xrd;
        //for unsolvent users
        if borrow_amount * (1 + liquidation_bonus) > deposit_amount {
            max_liquidation_percent = Decimal::ONE;
        }
        //TO DO: REMOVE
        /*if asset_borrow_amount < asset_min_liquidate_value {
            asset_borrow_amount *= *cost_of_repaid_asset_in_terms_of_xrd;
            max_repayment = asset_borrow_amount;
        } else {*/
        let max_repayment = max_liquidation_percent * borrow_amount;
        //}
        amount *= *cost_of_repaid_asset_in_terms_of_xrd;
        if amount > borrowable_amount_in_terms_of_xrd {
            panic!("Amount is greater than max borrowable amount");
        }
        liquidated_user_deposit_balance *= *cost_of_deposit_asset_in_terms_of_xrd;

        if amount > max_repayment {
            panic!("Amount is greater than max repayment");
        }
        let max_liquidating_amount: Decimal;
        if liquidated_user_deposit_balance == Decimal::ZERO {
            max_liquidating_amount = asset_borrow_amount;
        } else {
            max_liquidating_amount = liquidated_user_deposit_balance
                / ((liquidation_bonus * liquidation_reserve_factor)
                    + (1 + liquidation_bonus * (1 - liquidation_reserve_factor)));
        }
        if amount > max_liquidating_amount {
            panic!("Amount is greater than max liquidating amount");
        }

        let interest = self.on_liquidate_repay(
            amount,
            repaid_asset_address,
            *cost_of_repaid_asset_in_terms_of_xrd,
            sb_price,
        );

        //unsolvent user
        /*if borrow_amount > deposit_amount {
            liquidation_reserve_factor = Decimal::ONE;
        }*/
        // Platform is getting the liquidation fee
        let mut platform_bonus = amount * liquidation_bonus * liquidation_reserve_factor;
        if liquidated_user_deposit_balance == Decimal::ZERO {
            platform_bonus = Decimal::ZERO;
        }
        // Liquidator is getting liquidation reward and possible change from repaiment
        let mut reward = amount * (1 + liquidation_bonus * (1 - liquidation_reserve_factor));
        reward /= *cost_of_deposit_asset_in_terms_of_xrd;
        platform_bonus /= *cost_of_deposit_asset_in_terms_of_xrd;
        liquidated_user_deposit_balance /= *cost_of_deposit_asset_in_terms_of_xrd;
        let users_new_deposit_balance = liquidated_user_deposit_balance - reward - platform_bonus;
        self.update_deposit(deposit_asset_address, users_new_deposit_balance);
        let mut decreased_amount = amount - interest;
        decreased_amount /= *cost_of_repaid_asset_in_terms_of_xrd;
        (reward, platform_bonus, decreased_amount)
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

    pub fn calculate_total_collateral_and_loan(
        &mut self,
        pool_parameters: &HashMap<ResourceAddress, PoolParameters>,
        prices: HashMap<ResourceAddress, Decimal>,
    ) -> (Decimal, Decimal) {
        let mut user_collateral_sum: Decimal = 0.into();
        let mut user_loan_sum: Decimal = 0.into();

        // Iterate over each asset and calculate the amount of collateral and loan available from each
        for (asset_address, parameters) in pool_parameters {
            let cost_of_asset_in_terms_of_xrd = prices.get(asset_address).unwrap();
            let sr_deposit_balance = self.get_deposit(asset_address.clone());
            if sr_deposit_balance != Decimal::ZERO {
                let sd_price = parameters.deposit_balance / parameters.sr_deposit_balance;
                let ltv_ratio = &parameters.ltv_ratio;

                let asset_value_in_xrd =
                    sr_deposit_balance * sd_price * *cost_of_asset_in_terms_of_xrd;
                let asset_collateral = asset_value_in_xrd * *ltv_ratio;
                user_collateral_sum += asset_collateral;
            }
            let sr_borrow_balance = self.get_borrow(asset_address.clone());
            if sr_borrow_balance != Decimal::ZERO {
                let sb_price = parameters.borrow_balance / parameters.sr_borrow_balance;
                let asset_loan = sr_borrow_balance * sb_price * *cost_of_asset_in_terms_of_xrd;
                user_loan_sum += asset_loan;
            }
        }
        (user_collateral_sum, user_loan_sum)
    }

    pub fn get_loan_limit_used(
        &mut self,
        pool_parameters: &HashMap<ResourceAddress, PoolParameters>,
        prices: HashMap<ResourceAddress, Decimal>,
    ) -> Decimal {
        let collateral_and_loan = self.calculate_total_collateral_and_loan(pool_parameters, prices);
        let deposit = collateral_and_loan.0;
        let loan = collateral_and_loan.1;
        if loan == Decimal::ZERO {
            Decimal::ZERO
        } else {
            loan / deposit
        }
    }

    pub fn get_deposit_and_borrow_balance_in_xrd(
        &mut self,
        pool_parameters: &HashMap<ResourceAddress, PoolParameters>,
        prices: &HashMap<ResourceAddress, Decimal>,
    ) -> (Decimal, Decimal) {
        let mut deposit = Decimal::ZERO;
        let mut borrow = Decimal::ZERO;
        (self.deposits.clone())
            .into_iter()
            .for_each(|(_key, value)| {
                let parameters = pool_parameters.get(&_key).unwrap().clone();
                let cost_of_asset_in_terms_of_xrd = prices.get(&_key).unwrap();
                let sd_price = parameters.deposit_balance / parameters.sr_deposit_balance;

                let balance = value * sd_price;
                deposit += balance * *cost_of_asset_in_terms_of_xrd;
            });
        (self.borrows.clone())
            .into_iter()
            .for_each(|(_key, value)| {
                let parameters = pool_parameters.get(&_key).unwrap().clone();
                let cost_of_asset_in_terms_of_xrd = prices.get(&_key).unwrap();
                let sb_price = parameters.borrow_balance / parameters.sr_borrow_balance;

                let balance = value * sb_price;
                borrow += balance * *cost_of_asset_in_terms_of_xrd;
            });
        (deposit, borrow)
    }

    fn update_map(
        map: &mut IndexMap<ResourceAddress, Decimal>,
        key: ResourceAddress,
        units: Decimal,
    ) {
        map.insert(key, units);
    }
}
