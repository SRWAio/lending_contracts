use crate::resources::*;
use scrypto::prelude::*;

#[blueprint]
mod lending_protocol {

    extern_blueprint! {
    // import the Pool package from the ledger using its package address
    "package_rdx1pkz03xm6yfyuy6ua66w4ypvmpg4dtyxq0hxc6h0nvmzz9dklnf3d73",
    Pool {
        fn take(&mut self, amount: Decimal, reserve: Decimal)  -> Bucket;
        fn put(&mut self, bucket: Bucket, reserve: Decimal) ;
        }
    }
    extern_blueprint! {
    // import the PriceORacle package from the ledger using its package address
    "package_rdx1pkz03xm6yfyuy6ua66w4ypvmpg4dtyxq0hxc6h0nvmzz9dklnf3d73",
    PriceOracle {
        // Blueprint Functions
        /*fn instantiate_owned(price: Decimal, component_address: ComponentAddress) -> Owned<Lending>;
        fn instantiate_global(price: Decimal) -> ( Global<Lending>, Bucket); */

        // Component Methods
        fn get_price(&mut self, res_addr: ResourceAddress) -> Decimal;
        fn get_price_in_xrd(&mut self, res_addr: ResourceAddress) -> Decimal;
        //fn get_prices_in_xrd(&mut self) -> HashMap<ResourceAddress, Decimal>;
        //fn get_all_prices(&mut self) -> HashMap<ResourceAddress, Decimal>;
        //fn insert_to_liquidity_pool(&mut self, asset: Bucket);

        }
    }

    enable_method_auth! {
        // decide which methods are public and which are restricted to certain roles
        roles {
            admin => updatable_by: [admin];
        }
        ,methods {
            approve_admin_functions => restrict_to: [admin];
            mint_admin_badge => restrict_to: [admin];
        }
    }

    struct LendingProtocol {
        assets_in_use: IndexSet<ResourceAddress>,
        pool_address: Global<Pool>,
        oracle_address: Global<PriceOracle>,
        admin_signature_check: HashMap<NonFungibleLocalId, bool>,
        /// A counter for ID generation
        admin_badge_id_counter: u64,
        admin_badge_address: ResourceAddress,
        /// User badge resource manager
        user_resource_manager: ResourceManager,
        /// Admin badge resource manager
        admin_resource_manager: ResourceManager,
        /// Protocol badge resource manager
        protocol_resource_manager: ResourceManager,
    }

    impl LendingProtocol {
        pub fn instantiate() {}

        pub fn deposit() {}
        pub fn withdraw() {}
        pub fn borrow() {}
        pub fn repay() {}
        pub fn liquidate() {}

        pub fn approve_admin_functions(&mut self, admin_badge: Proof) {
            let manager = ResourceManager::from(admin_badge.resource_address());
            let admin_id = admin_badge
                .check(manager.address())
                .as_non_fungible()
                .non_fungible_local_id();
            self.admin_signature_check.insert(admin_id, true);
        }

        pub fn mint_admin_badge(&mut self, admin_badge: Proof) -> NonFungibleBucket {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }

            // Mint
            let resource_manager = NonFungibleResourceManager::from(admin_badge.resource_address());
            let admin_badge_id_counter = self.admin_badge_id_counter;
            let new_id = admin_badge_id_counter + 1;
            let admin_name = "Admin ".to_string() + &new_id.to_string();
            let new_admin_badge = resource_manager.mint_non_fungible(
                &NonFungibleLocalId::integer(new_id),
                AdminBadge { name: admin_name },
            );
            self.admin_badge_id_counter += 1;
            self.admin_signature_check = HashMap::new();
            new_admin_badge
        }

        /*pub fn take_protocol_badge(&mut self) -> Bucket {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.admin_signature_check = HashMap::new();
            lending_badge
        }*/

        fn is_approved_by_admins(&mut self) -> bool {
            let singature_count = self.admin_signature_check.len();
            if singature_count < 3 {
                false
            } else {
                true
            }
        }
    }
}
