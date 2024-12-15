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
        admin_rule: AccessRule,
        assets_in_use: IndexSet<ResourceAddress>,
        oracle_address: Global<PriceOracle>,
        admin_signature_check: HashMap<NonFungibleLocalId, bool>,
        /// A counter for ID generation
        admin_badge_id_counter: u64,
        admin_badge_address: ResourceAddress,
        /// User badge resource manager
        user_resource_manager: ResourceManager,
        // Admin badge resource manager
        //admin_resource_manager: ResourceManager,
        // Protocol badge resource manager
        //protocol_resource_manager: ResourceManager,
    }

    impl LendingProtocol {
        pub fn instantiate(oracle_address: Global<PriceOracle>) -> NonFungibleBucket {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let component_rule = rule!(require(global_caller(protocol_component_address)));

            // * Create admin badge * //

            // Get address reservation for the admin badge resource address
            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule = rule!(require_amount(dec!(4), admin_badge_address));

            // Moderator will be able to update operating status if the last update was not done by an admin
            //let moderator_rule = rule!(require_amount(dec!(2), admin_badge_address));

            let admin_badge =
                create_admin_badge(admin_rule.clone(), admin_badge_address_reservation);

            let user_resource_manager =
                create_user_resource_manager(admin_rule.clone(), component_rule.clone());

            // *  Instantiate our component with the previously created resources and addresses * //
            Self {
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                admin_signature_check: HashMap::new(),
                oracle_address,
                admin_badge_address: admin_badge.resource_address(),
                admin_badge_id_counter: 5,
                assets_in_use: IndexSet::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(protocol_component_address_reservation)
            .roles(roles! {
                admin => admin_rule.clone();
            })
            .metadata(metadata!(
                roles {
                    metadata_setter => admin_rule.clone();
                    metadata_setter_updater => rule!(deny_all);
                    metadata_locker => admin_rule;
                    metadata_locker_updater => rule!(deny_all);
                }
            ))
            .globalize();

            admin_badge
        }

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
