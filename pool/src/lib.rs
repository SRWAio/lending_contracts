use scrypto::prelude::*;

#[blueprint]
mod pool {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            take => restrict_to :[admin];
            put => restrict_to :[admin];
            get_pool_balances => PUBLIC;
        }
    }

    struct Pool {
        // The liquidity pool
        liquidity_pool: Vault,
        deposit: Decimal,
        sd_balance: Decimal,
        borrow: Decimal,
        sb_balance: Decimal,
        reserve: Decimal,
        updated_at: u64,
    }

    impl Pool {
        pub fn instantiate(
            admin_rule: AccessRule,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress) {
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Pool::blueprint_id());
            // Instantiate a pool_component
            let pool_component = Self {
                liquidity_pool: Vault::new(resource_address),
                deposit: Decimal::zero(),
                sd_balance: Decimal::zero(),
                borrow: Decimal::zero(),
                sb_balance: Decimal::zero(),
                reserve: Decimal::zero(),
                updated_at: 0,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .roles(roles!(
                admin => admin_rule;
            ))
            .with_address(address_reservation)
            .globalize();
            (pool_component, component_address)
        }

        pub fn take(
            &mut self,
            amount: Decimal,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,
        ) -> Bucket {
            self.borrow = borrow;
            self.deposit = deposit;
            self.sd_balance = sd_balance;
            self.sb_balance = sb_balance;
            self.reserve = reserve;
            self.updated_at = Runtime::current_epoch().number();
            self.liquidity_pool
                .take_advanced(amount, WithdrawStrategy::Rounded(RoundingMode::ToZero))
        }

        pub fn put(
            &mut self,
            bucket: Bucket,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,
        ) {
            self.borrow = borrow;
            self.deposit = deposit;
            self.sd_balance = sd_balance;
            self.sb_balance = sb_balance;
            self.reserve = reserve;
            self.updated_at = Runtime::current_epoch().number();
            self.liquidity_pool.put(bucket)
        }

        pub fn get_pool_balances(&mut self) -> (Decimal, Decimal, Decimal, Decimal, Decimal) {
            (
                self.deposit,
                self.sd_balance,
                self.borrow,
                self.sb_balance,
                self.reserve,
            )
        }
    }
}
