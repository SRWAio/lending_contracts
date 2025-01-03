use scrypto::prelude::*;

#[blueprint]
mod pool {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            deposit => restrict_to :[admin];
            withdraw => restrict_to :[admin];
            take => restrict_to :[admin];
            put => restrict_to :[admin];
        }
    }

    struct Pool {
        // The liquidity pool
        liquidity_pool: Vault,
        deposit: Decimal,
        sr_deposit: Decimal,
        borrow: Decimal,
        sr_borrow: Decimal,
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

            let component_rule = rule!(require(global_caller(component_address)));

            // Instantiate a pool_component
            let pool_component = Self {
                liquidity_pool: Vault::new(resource_address),
                deposit: Decimal::zero(),
                sr_deposit: Decimal::zero(),
                sr_borrow: Decimal::zero(),
                borrow: Decimal::zero(),
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

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn deposit(&mut self) {
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            info!("Deposit initiated.");
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn withdraw(&mut self) {
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            info!("Deposit initiated.");
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn take(
            &mut self,
            amount: Decimal,
            deposit: Decimal,
            sr_deposit: Decimal,
            borrow: Decimal,
            sr_borrow: Decimal,
            reserve: Decimal,
        ) -> Bucket {
            self.borrow = borrow;
            self.deposit = deposit;
            self.sr_deposit = sr_deposit;
            self.sr_borrow = sr_borrow;
            self.reserve = reserve;
            self.updated_at = Runtime::current_epoch().number();
            self.liquidity_pool.take(amount)
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn put(
            &mut self,
            bucket: Bucket,
            deposit: Decimal,
            sr_deposit: Decimal,
            borrow: Decimal,
            sr_borrow: Decimal,
            reserve: Decimal,
        ) {
            self.borrow = borrow;
            self.deposit = deposit;
            self.sr_deposit = sr_deposit;
            self.sr_borrow = sr_borrow;
            self.reserve = reserve;
            self.updated_at = Runtime::current_epoch().number();
            self.liquidity_pool.put(bucket)
        }
    }
}
