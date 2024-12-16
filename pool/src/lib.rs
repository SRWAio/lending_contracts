use scrypto::prelude::*;

#[blueprint]
mod pool {
    enable_method_auth! {
        roles {
            admin => updatable_by: [];
        },
        methods {
            deposit => restrict_to :[admin];
            take => restrict_to :[admin];
            put => restrict_to :[admin];
        }
    }

    struct Pool {
        // The liquidity pool
        liquidity_pool: Vault,
        deposit: Decimal,
        borrow: Decimal,
        reserve: Decimal,
    }

    impl Pool {
        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate(
            admin_rule: AccessRule,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress) {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let (address_reservation, component_address) =
                Runtime::allocate_component_address(Pool::blueprint_id());

            let component_rule = rule!(require(global_caller(component_address)));

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            let pool_component = Self {
                liquidity_pool: Vault::new(resource_address),
                deposit: Decimal::zero(),
                borrow: Decimal::zero(),
                reserve: Decimal::zero(),
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
        pub fn take(&mut self, amount: Decimal, reserve: Decimal) -> Bucket {
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            self.borrow += amount;
            self.reserve += reserve;
            self.liquidity_pool.take(amount)
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn put(&mut self, bucket: Bucket, reserve: Decimal) {
            self.deposit += bucket.amount();
            self.reserve += reserve;
            self.liquidity_pool.put(bucket)

            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
        }
    }
}
