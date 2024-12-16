use crate::resources::*;
use scrypto::prelude::*;

#[blueprint]
mod lending_protocol {

    extern_blueprint! {
    // import the Pool package from the ledger using its package address
    "package_sim1pk3cmat8st4ja2ms8mjqy2e9ptk8y6cx40v4qnfrkgnxcp2krkpr92",
    Pool {
        fn instantiate(
            admin_rule: AccessRule,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress);
        fn deposit(&mut self);
        fn take(&mut self, amount: Decimal, reserve: Decimal)  -> Bucket;
        fn put(&mut self, bucket: Bucket, reserve: Decimal);
        }
    }
    /*extern_blueprint! {
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
    }*/

    enable_method_auth! {
        // decide which methods are public and which are restricted to certain roles
        roles {
            admin => updatable_by: [admin];
        }
        ,methods {
            approve_admin_functions => restrict_to: [admin];
            mint_admin_badge => restrict_to: [admin];
            take_protocol_badge => restrict_to: [admin];
            create_pool => restrict_to: [admin];
            deposit =>  PUBLIC;
            insert_pool_component =>  PUBLIC;
        }
    }

    struct LendingProtocol {
        pools: HashMap<ResourceAddress, Global<Pool>>,
        protocol_badge: NonFungibleVault,
        admin_rule: AccessRule,
        protocol_rule: AccessRule,
        assets_in_use: IndexSet<ResourceAddress>,
        //oracle_address: Global<PriceOracle>,
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
        pub fn instantiate(/*oracle_address: Global<PriceOracle>*/) -> NonFungibleBucket {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let component_rule = rule!(require(global_caller(protocol_component_address)));

            // * Create admin badge * //

            // Get address reservation for the admin badge resource address
            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule: AccessRule = rule!(require(admin_badge_address));

            // Moderator will be able to update operating status if the last update was not done by an admin
            //let moderator_rule = rule!(require_amount(dec!(2), admin_badge_address));

            let admin_badge =
                create_admin_badge(admin_rule.clone(), admin_badge_address_reservation);

            let protocol_badge = create_protocol_badge(admin_rule.clone());
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));
            let user_resource_manager =
                create_user_resource_manager(admin_rule.clone(), component_rule.clone());

            // *  Instantiate our component with the previously created resources and addresses * //
            Self {
                protocol_badge: NonFungibleVault::with_bucket(protocol_badge),
                pools: HashMap::new(),
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                protocol_rule: protocol_rule,
                admin_signature_check: HashMap::new(),
                //oracle_address,
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

        pub fn instantiate_new_version(
            //oracle_address: Global<PriceOracle>,
            protocol_badge: NonFungibleBucket,
            admin_badge_address: ResourceAddress,
        ) {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let component_rule = rule!(require(global_caller(protocol_component_address)));
            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule: AccessRule = rule!(require(admin_badge_address));
            let user_resource_manager =
                create_user_resource_manager(admin_rule.clone(), component_rule.clone());
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));

            // *  Instantiate our component with the previously created resources and addresses * //
            Self {
                protocol_badge: NonFungibleVault::with_bucket(protocol_badge),
                pools: HashMap::new(),
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                admin_signature_check: HashMap::new(),
                protocol_rule,
                //oracle_address,
                admin_badge_address,
                admin_badge_id_counter: 5,
                //assets_in_use,
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
        }

        pub fn create_pool(
            &mut self,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress) {
            let pool_component_address =
                Blueprint::<Pool>::instantiate(self.protocol_rule.clone(), resource_address);
            self.pools
                .insert(resource_address, pool_component_address.0);
            pool_component_address
        }

        pub fn deposit(&mut self, resource_address: ResourceAddress) {
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            self.protocol_badge
                .authorize_with_non_fungibles(&non_fungible_local_ids, || pool.deposit());
        }

        pub fn insert_pool_component(
            &mut self,
            resource_address: ResourceAddress,
            pool_component: Global<Pool>,
        ) {
            self.pools.insert(resource_address, pool_component);
        }
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

        pub fn take_protocol_badge(&mut self) -> NonFungibleBucket {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.admin_signature_check = HashMap::new();
            let protocol_badge = self.protocol_badge.take(1);
            protocol_badge
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
