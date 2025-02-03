use crate::calculations::*;
use crate::resources::*;
use crate::user::UserData;
use scrypto::prelude::*;

#[blueprint]
mod lending_protocol {
    use crate::pool_parameters::PoolParameters;

    extern_blueprint! {
    // import the Pool package from the ledger using its package address
    "package_tdx_2_1pk2dgttfuun4qf7shskfx53pqs6um6psaefy4203guwr2fp9j83n83",
    Pool {
        fn instantiate(
            admin_rule: AccessRule,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress);
        fn deposit(&mut self);
        /*fn withdraw(&mut self) -> Bucket;
        fn borrow(&mut self) -> Bucket;*/
        fn take(&mut self, amount: Decimal,
            deposit: Decimal,
            sr_deposit: Decimal,
            borrow: Decimal,
            sr_borrow: Decimal,
            reserve: Decimal,)  -> Bucket;
        fn put(&mut self, bucket: Bucket,
            deposit: Decimal,
            sr_deposit: Decimal,
            borrow: Decimal,
            sr_borrow: Decimal,
            reserve: Decimal,);
        }
    }
    extern_blueprint! {
    // import the PriceORacle package from the ledger using its package address
    "package_tdx_2_1ph0hwlqmde3ht29pzy5qehqflvjfrtty4lgyvwhhqp589e0v0qhtke",
    PriceOracle {
        // Component Methods
        fn get_price(&mut self, res_addr: ResourceAddress) -> Decimal;
        fn get_price_in_xrd(&mut self, res_addr: ResourceAddress) -> Decimal;
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
            take_protocol_badge => restrict_to: [admin];
            create_pool => restrict_to: [admin];
            create_user_and_deposit =>  PUBLIC;
            deposit =>  PUBLIC;
            withdraw =>  PUBLIC;
            borrow =>  PUBLIC;
            repay => PUBLIC;
            liquidate => restrict_to: [admin];
            insert_pool_component =>  PUBLIC;
            update_pool_parameters => restrict_to: [admin];
        }
    }

    struct LendingProtocol {
        pools: HashMap<ResourceAddress, Global<Pool>>,
        protocol_badge: NonFungibleVault,
        admin_rule: AccessRule,
        component_rule: AccessRule,
        protocol_rule: AccessRule,
        assets_in_use: IndexSet<ResourceAddress>,
        oracle_address: Global<PriceOracle>,
        admin_signature_check: HashMap<NonFungibleLocalId, bool>,
        /// A counter for ID generation
        admin_badge_id_counter: u64,
        admin_badge_address: ResourceAddress,
        /// User badge resource manager
        user_resource_manager: NonFungibleResourceManager,
        // Admin badge resource manager
        //admin_resource_manager: ResourceManager,
        // Protocol badge resource manager
        //protocol_resource_manager: ResourceManager,
        pool_parameters: HashMap<ResourceAddress, PoolParameters>,
        ltv_ratios: HashMap<ResourceAddress, Decimal>,
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
                component_rule: component_rule.clone(),
                protocol_rule: protocol_rule,
                admin_signature_check: HashMap::new(),
                admin_badge_address: admin_badge.resource_address(),
                admin_badge_id_counter: 5,
                assets_in_use: IndexSet::new(),
                pool_parameters: HashMap::new(),
                oracle_address,
                ltv_ratios: HashMap::new(),
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
            oracle_address: Global<PriceOracle>,
            protocol_badge: NonFungibleBucket,
            user_badge_address: ResourceAddress,
            admin_badge_address: ResourceAddress,
        ) {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let component_rule = rule!(require(global_caller(protocol_component_address)));
            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule: AccessRule = rule!(require(admin_badge_address));
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));
            let user_resource_manager: NonFungibleResourceManager = user_badge_address.into();
            // *  Instantiate our component with the previously created resources and addresses * //
            Self {
                protocol_badge: NonFungibleVault::with_bucket(protocol_badge),
                pools: HashMap::new(),
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                component_rule: component_rule.clone(),
                admin_signature_check: HashMap::new(),
                protocol_rule,
                admin_badge_address,
                admin_badge_id_counter: 5,
                assets_in_use: IndexSet::new(),
                pool_parameters: HashMap::new(),
                oracle_address,
                ltv_ratios: HashMap::new(),
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

        pub fn insert_pool_component(
            &mut self,
            resource_address: ResourceAddress,
            pool_component: Global<Pool>,
        ) {
            self.pools.insert(resource_address, pool_component);
        }

        pub fn create_pool(
            &mut self,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress) {
            let pool_component_address =
                Blueprint::<Pool>::instantiate(self.protocol_rule.clone(), resource_address);
            self.pools
                .insert(resource_address, pool_component_address.0);
            self.ltv_ratios.insert(resource_address, dec!("0.5"));
            let now = Runtime::current_epoch().number();

            let data = PoolParameters {
                balances_updated_at: now,
                base: dec!("0.001"),
                min_collateral_ratio: Decimal::one(),
                max_borrow_percent: dec!("0.1"),
                max_liquidation_percent: dec!("0.5"),
                liquidation_bonus: dec!("0.1"),
                ltv_ratio: dec!("0.5"),
                multiplier: dec!("10"),
                base_multiplier: dec!("0.03"),
                reserve_factor: Decimal::zero(),
                kink: dec!("0.7"),
                liquidation_reserve_factor: dec!("0.2"),
                min_liquidable_value: dec!("6000"),
                deposit_locked: false,
                borrow_locked: false,
                withdraw_locked: false,
                repay_locked: false,
                pool_reserve: dec!("0.2"),
                deposit_limit: dec!("10000"),
                borrow_balance: Decimal::zero(),
                deposit_balance: Decimal::zero(),
                reserve_balance: Decimal::zero(),
                sr_deposit_balance: Decimal::zero(),
                sr_borrow_balance: Decimal::zero(),
            };

            self.pool_parameters.insert(resource_address, data);
            pool_component_address
        }

        pub fn create_user_and_deposit(&mut self, asset: Bucket) -> NonFungibleBucket {
            let resource_address = asset.resource_address();
            let asset_amount = asset.amount();
            let pool_parameters = self.pool_parameters.get(&resource_address).unwrap().clone();

            let deposit_locked = pool_parameters.deposit_locked;
            if deposit_locked {
                panic!("Depositing is locked for now!");
            }
            let pool_deposit_limit = pool_parameters.deposit_limit;
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sr_deposit_balance = pool_parameters.sr_deposit_balance;

            if pool_deposit_limit > Decimal::ZERO {
                let asset_price = self.oracle_address.get_price(resource_address);
                let current_deposit_balance =
                    (asset_total_deposit_balance + asset.amount()) * asset_price;
                if current_deposit_balance > pool_deposit_limit {
                    panic!("Deposit limit is {} .", pool_deposit_limit / asset_price);
                }
            }
            let utilisation =
                get_utilisation(asset_total_deposit_balance, asset_total_borrow_balance);
            let borrow_rate = calculate_borrow_rate(
                pool_parameters.multiplier,
                pool_parameters.base_multiplier,
                pool_parameters.base,
                pool_parameters.kink,
                utilisation,
            );
            let borrow_apr = calculate_borrow_apr(borrow_rate, pool_parameters.balances_updated_at);
            let interests = calculate_interests(
                asset_total_borrow_balance,
                borrow_apr,
                pool_parameters.reserve_factor,
            );
            asset_total_deposit_balance += interests.2;

            let sd_interest = calculate_sd_interest(
                asset.amount(),
                asset_total_deposit_balance,
                sr_deposit_balance,
            );
            sr_deposit_balance += sd_interest;
            let mut user_count = match self.user_resource_manager.total_supply() {
                Some(value) => value,
                None => Decimal::zero(),
            };
            user_count += Decimal::one();
            let user_id_converted: u64 = user_count.try_into().unwrap();
            let user_id = NonFungibleLocalId::Integer(user_id_converted.into());
            let now = Runtime::current_epoch().number();
            let mut deposits = IndexMap::new();
            deposits.insert(resource_address, sd_interest);
            let mut borrows = IndexMap::new();
            borrows.insert(resource_address, Decimal::zero());

            let data = UserData {
                name: "User Badge".into(),
                image_url: "https://demo.srwa.io/images/badge.png".into(),
                deposits,
                borrows,
                minted_at: now,
                updated_at: now,
            };
            let user = self.user_resource_manager.mint_non_fungible(&user_id, data);
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += asset_amount;
            self.update_pool_balances(
                resource_address,
                asset_total_deposit_balance,
                sr_deposit_balance,
                asset_total_borrow_balance,
                pool_parameters.sr_borrow_balance,
                asset_total_reserve_balance,
            );
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            self.protocol_badge
                .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                    pool.put(
                        asset,
                        asset_total_deposit_balance,
                        sr_deposit_balance,
                        asset_total_borrow_balance,
                        pool_parameters.sr_borrow_balance,
                        asset_total_reserve_balance,
                    )
                });
            user
        }

        pub fn deposit(&mut self, asset: Bucket, user_badge: Proof) {
            let resource_address = asset.resource_address();
            let asset_amount = asset.amount();
            let user_badge_resource_address = user_badge.resource_address();
            let pool_parameters = self.pool_parameters.get(&resource_address).unwrap().clone();

            let deposit_locked = pool_parameters.deposit_locked;
            if deposit_locked {
                panic!("Depositing is locked for now!");
            }
            let pool_deposit_limit = pool_parameters.deposit_limit;
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sr_deposit_balance = pool_parameters.sr_deposit_balance;

            if pool_deposit_limit > Decimal::ZERO {
                let asset_price = self.oracle_address.get_price(resource_address);
                let current_deposit_balance =
                    (asset_total_deposit_balance + asset.amount()) * asset_price;
                if current_deposit_balance > pool_deposit_limit {
                    panic!("Deposit limit is {} .", pool_deposit_limit / asset_price);
                }
            }
            let utilisation =
                get_utilisation(asset_total_deposit_balance, asset_total_borrow_balance);
            let borrow_rate = calculate_borrow_rate(
                pool_parameters.multiplier,
                pool_parameters.base_multiplier,
                pool_parameters.base,
                pool_parameters.kink,
                utilisation,
            );
            let borrow_apr = calculate_borrow_apr(borrow_rate, pool_parameters.balances_updated_at);
            let interests = calculate_interests(
                asset_total_borrow_balance,
                borrow_apr,
                pool_parameters.reserve_factor,
            );
            asset_total_deposit_balance += interests.2;

            let sd_interest = calculate_sd_interest(
                asset.amount(),
                asset_total_deposit_balance,
                sr_deposit_balance,
            );
            sr_deposit_balance += sd_interest;
            let manager_address = self.user_resource_manager.address();
            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += asset_amount;

            self.update_pool_balances(
                resource_address,
                asset_total_deposit_balance,
                sr_deposit_balance,
                asset_total_borrow_balance,
                pool_parameters.sr_borrow_balance,
                asset_total_reserve_balance,
            );
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            user.on_deposit(resource_address, sd_interest);
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "deposits",
                user.deposits,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "updated_at",
                Runtime::current_epoch().number(),
            );
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            self.protocol_badge
                .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                    pool.put(
                        asset,
                        asset_total_deposit_balance,
                        sr_deposit_balance,
                        asset_total_borrow_balance,
                        pool_parameters.sr_borrow_balance,
                        asset_total_reserve_balance,
                    )
                });
        }

        pub fn withdraw(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
            user_badge: Proof,
        ) -> Bucket {
            //Get user badge
            let user_badge_resource_address = user_badge.resource_address();
            let manager_address = self.user_resource_manager.address();

            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            let pool_parameters = self.pool_parameters.get(&resource_address).unwrap().clone();
            let withdraw_locked = pool_parameters.withdraw_locked;
            if withdraw_locked {
                panic!("Withdrawing is locked for now!");
            }

            let withdrawable_amount = self.borrowable_amount(
                pool_parameters.deposit_balance,
                pool_parameters.borrow_balance,
                pool_parameters.reserve_balance,
                pool_parameters.pool_reserve,
            );
            if withdrawable_amount < amount {
                panic!("Max withdrawal amount is {}: ", withdrawable_amount);
            }

            let asset_ltv_ratio = pool_parameters.ltv_ratio;
            let mut prices = HashMap::new();
            for (&res_address, &_ratio) in &self.ltv_ratios {
                let mut price_in_xrd = Decimal::ONE;
                if res_address != XRD {
                    price_in_xrd = self.oracle_address.get_price_in_xrd(res_address);
                }
                prices.insert(res_address, price_in_xrd);
            }
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let user_available_collateral =
                user.calculate_total_collateral(&self.pool_parameters, prices.clone());
            let withdrawable_amount_in_xrd = user_available_collateral / asset_ltv_ratio;
            let cost_of_asset_in_terms_of_xrd = prices.get(&resource_address).unwrap();
            let withdrawable_amount = withdrawable_amount_in_xrd / *cost_of_asset_in_terms_of_xrd;

            if amount > withdrawable_amount {
                panic!("Max withrawal amount is {}: ", withdrawable_amount);
            }
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sr_deposit_balance = pool_parameters.sr_deposit_balance;

            let utilisation =
                get_utilisation(asset_total_deposit_balance, asset_total_borrow_balance);
            let borrow_rate = calculate_borrow_rate(
                pool_parameters.multiplier,
                pool_parameters.base_multiplier,
                pool_parameters.base,
                pool_parameters.kink,
                utilisation,
            );
            let borrow_apr = calculate_borrow_apr(borrow_rate, pool_parameters.balances_updated_at);
            let interests = calculate_interests(
                asset_total_borrow_balance,
                borrow_apr,
                pool_parameters.reserve_factor,
            );
            asset_total_deposit_balance += interests.2;
            let sd_interest =
                calculate_sd_interest(amount, asset_total_deposit_balance, sr_deposit_balance);
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance -= amount;
            sr_deposit_balance -= sd_interest;
            self.update_pool_balances(
                resource_address,
                asset_total_deposit_balance,
                sr_deposit_balance,
                asset_total_borrow_balance,
                pool_parameters.sr_borrow_balance,
                asset_total_reserve_balance,
            );

            user.on_withdraw(resource_address, sd_interest);
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "deposits",
                user.deposits,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "updated_at",
                Runtime::current_epoch().number(),
            );
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let withdrawn_asset =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            amount,
                            asset_total_deposit_balance,
                            sr_deposit_balance,
                            asset_total_borrow_balance,
                            pool_parameters.sr_borrow_balance,
                            asset_total_reserve_balance,
                        )
                    });
            withdrawn_asset
        }

        pub fn borrow(
            &mut self,
            asset_address: ResourceAddress,
            amount: Decimal,
            user_badge: Proof,
        ) -> Bucket {
            //Get user data
            let user_badge_resource_address = user_badge.resource_address();
            let manager_address = self.user_resource_manager.address();

            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            let pool_parameters = self.pool_parameters.get(&asset_address).unwrap().clone();
            let borrow_locked = pool_parameters.borrow_locked;
            if borrow_locked {
                panic!("Withdrawing is locked for now!");
            }

            let borrowable_amount = self.borrowable_amount(
                pool_parameters.deposit_balance,
                pool_parameters.borrow_balance,
                pool_parameters.reserve_balance,
                pool_parameters.pool_reserve,
            );
            if borrowable_amount < amount {
                panic!("Max borrow amount is {}: ", borrowable_amount);
            }

            let mut prices = HashMap::new();
            for (&res_address, &_ratio) in &self.ltv_ratios {
                let mut price_in_xrd = Decimal::ONE;
                if res_address != XRD {
                    price_in_xrd = self.oracle_address.get_price_in_xrd(res_address);
                }
                prices.insert(res_address, price_in_xrd);
            }
            let cost_of_asset_in_terms_of_xrd = prices.get(&asset_address).unwrap();

            let borrow_amount_in_terms_of_xrd = amount * *cost_of_asset_in_terms_of_xrd;
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let user_available_collateral =
                user.calculate_total_collateral(&self.pool_parameters, prices.clone());
            assert!(
                user_available_collateral >= borrow_amount_in_terms_of_xrd,
                "[borrow_asset][POOL] User does not have enough collateral. Requested loan with \
                  value of `{:?}` XRD but only has `{:?}` XRD of available collateral.",
                borrow_amount_in_terms_of_xrd,
                user_available_collateral
            );
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sr_borrow_balance = pool_parameters.sr_borrow_balance;
            let utilisation =
                get_utilisation(asset_total_deposit_balance, asset_total_borrow_balance);
            let borrow_rate = calculate_borrow_rate(
                pool_parameters.multiplier,
                pool_parameters.base_multiplier,
                pool_parameters.base,
                pool_parameters.kink,
                utilisation,
            );
            let borrow_apr = calculate_borrow_apr(borrow_rate, pool_parameters.balances_updated_at);
            let interests = calculate_interests(
                asset_total_borrow_balance,
                borrow_apr,
                pool_parameters.reserve_factor,
            );
            asset_total_borrow_balance += interests.0;
            let sb_interest =
                calculate_sb_interest(amount, asset_total_borrow_balance, sr_borrow_balance);
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;
            asset_total_borrow_balance += amount;
            sr_borrow_balance += sb_interest;
            self.update_pool_balances(
                asset_address,
                asset_total_deposit_balance,
                pool_parameters.sr_deposit_balance,
                asset_total_borrow_balance,
                sr_borrow_balance,
                asset_total_reserve_balance,
            );

            user.on_borrow(asset_address, sb_interest);
            let mut pool = self.pools.get(&asset_address).unwrap().clone();
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "borrows",
                user.borrows,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "updated_at",
                Runtime::current_epoch().number(),
            );
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let borrowed_asset =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            amount,
                            asset_total_deposit_balance,
                            pool_parameters.sr_deposit_balance,
                            asset_total_borrow_balance,
                            sr_borrow_balance,
                            asset_total_reserve_balance,
                        )
                    });

            borrowed_asset
        }

        pub fn repay(&mut self, mut repaid: Bucket, user_badge: Proof) -> Bucket {
            let user_badge_resource_address = user_badge.resource_address();
            let manager_address = self.user_resource_manager.address();

            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            let asset_address = repaid.resource_address();
            let pool_parameters = self.pool_parameters.get(&asset_address).unwrap().clone();
            let repay_locked = pool_parameters.repay_locked;
            if repay_locked {
                panic!("Withdrawing is locked for now!");
            }

            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sr_borrow_balance = pool_parameters.sr_borrow_balance;
            let utilisation =
                get_utilisation(asset_total_deposit_balance, asset_total_borrow_balance);
            let borrow_rate = calculate_borrow_rate(
                pool_parameters.multiplier,
                pool_parameters.base_multiplier,
                pool_parameters.base,
                pool_parameters.kink,
                utilisation,
            );
            let borrow_apr = calculate_borrow_apr(borrow_rate, pool_parameters.balances_updated_at);

            let interests = calculate_interests(
                asset_total_borrow_balance,
                borrow_apr,
                pool_parameters.reserve_factor,
            );
            asset_total_borrow_balance += interests.0;
            let sb_interest = calculate_sb_interest(
                repaid.amount(),
                asset_total_borrow_balance,
                sr_borrow_balance,
            );
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;
            asset_total_borrow_balance -= repaid.amount();
            sr_borrow_balance -= sb_interest;
            self.update_pool_balances(
                asset_address,
                asset_total_deposit_balance,
                pool_parameters.sr_deposit_balance,
                asset_total_borrow_balance,
                sr_borrow_balance,
                asset_total_reserve_balance,
            );
            let non_fungible_id: NonFungibleLocalId = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let to_return = user.on_repay(asset_address, sb_interest);
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "borrows",
                user.borrows,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "updated_at",
                Runtime::current_epoch().number(),
            );
            let return_bucket = repaid.take(to_return);
            let mut pool = self.pools.get(&asset_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            self.protocol_badge
                .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                    pool.put(
                        repaid,
                        asset_total_deposit_balance,
                        pool_parameters.sr_deposit_balance,
                        asset_total_borrow_balance,
                        sr_borrow_balance,
                        asset_total_reserve_balance,
                    )
                });

            return_bucket
        }

        pub fn liquidate(
            &mut self,
            user_id: Decimal,
            repaid: Bucket,
            deposited_asset: ResourceAddress,
        ) -> Bucket {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.admin_signature_check = HashMap::new();
            let repaid_resource_address = repaid.resource_address();
            let integer_user_id = user_id
                .to_string()
                .parse::<u64>()
                .expect("Invalid decimal value");
            let non_fungible_id = NonFungibleLocalId::Integer(integer_user_id.into());
            // Get the user that is targeted for liquidation
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            //TO DO: Calculate balance based on price
            let liquidated_user_deposit_balance: Decimal = user.get_deposit(deposited_asset);
            if liquidated_user_deposit_balance == Decimal::ZERO {
                panic!("User deposit balance of selected token is 0.");
            }
            let repaid_pool_parameters = self
                .pool_parameters
                .get(&repaid_resource_address)
                .unwrap()
                .clone();

            let mut prices = HashMap::new();
            for (&res_address, &_ratio) in &self.ltv_ratios {
                let mut price_in_xrd = Decimal::ONE;
                if res_address != XRD {
                    price_in_xrd = self.oracle_address.get_price_in_xrd(res_address);
                }
                prices.insert(res_address, price_in_xrd);
            }
            let loan_limit_used = user.get_loan_limit_used(&self.pool_parameters, prices.clone());

            if loan_limit_used == Decimal::ZERO {
                panic!("No borrow from the user");
            }
            let deposit_and_borrow_in_xrd =
                user.get_deposit_and_borrow_balance_in_xrd(&self.pool_parameters, &prices);
            let borrow_amount_in_xrd = deposit_and_borrow_in_xrd.1;

            let deposit_amount_in_xrd = deposit_and_borrow_in_xrd.0;

            let asset_min_liquidate_value = repaid_pool_parameters.min_liquidable_value;

            let asset_max_liquidation_percent = repaid_pool_parameters.max_liquidation_percent;

            let asset_liquidation_bonus = repaid_pool_parameters.liquidation_bonus;

            let asset_liquidation_reserve_factor =
                repaid_pool_parameters.liquidation_reserve_factor;

            let repaid_asset_total_deposit_balance = repaid_pool_parameters.deposit_balance;
            let repaid_asset_total_borrow_balance = repaid_pool_parameters.borrow_balance;
            let repaid_asset_total_sr_borrow_balance = repaid_pool_parameters.sr_borrow_balance;

            let asset_borrow_amount = user.get_borrow(repaid_resource_address);

            let lending_parameters = self.pool_parameters.get(&deposited_asset).unwrap().clone();
            let deposit_balance = lending_parameters.deposit_balance;
            let borrow_balance = lending_parameters.borrow_balance;
            let reserve_balance = lending_parameters.reserve_balance;
            let pool_reserve = lending_parameters.pool_reserve;

            let borrowable_amount = self.borrowable_amount(
                deposit_balance,
                borrow_balance,
                reserve_balance,
                pool_reserve,
            );

            let sb_price = calculate_token_price(
                repaid_asset_total_borrow_balance,
                repaid_asset_total_sr_borrow_balance,
            );
            // Do the liquidation calculations and update the liquidated users state
            let to_return_amounts = user.on_liquidate(
                repaid.amount(),
                repaid_resource_address,
                borrow_amount_in_xrd,
                deposit_amount_in_xrd,
                asset_max_liquidation_percent,
                asset_liquidation_bonus,
                asset_liquidation_reserve_factor,
                liquidated_user_deposit_balance,
                deposited_asset,
                prices,
                asset_min_liquidate_value,
                asset_borrow_amount,
                borrowable_amount,
                sb_price,
            );

            let reward = to_return_amounts.0;
            let platform_bonus = to_return_amounts.1;
            let decreased_amount = to_return_amounts.2;
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "deposits",
                user.deposits,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "borrows",
                user.borrows,
            );
            self.user_resource_manager.update_non_fungible_data(
                &non_fungible_id,
                "updated_at",
                Runtime::current_epoch().number(),
            );
            let mut pool = self.pools.get(&deposited_asset).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let new_total_deposit = deposit_balance - reward - platform_bonus;
            let new_total_sr_deposit =
                lending_parameters.sr_deposit_balance - reward - platform_bonus;
            let to_return_reward =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            reward,
                            new_total_deposit,
                            new_total_sr_deposit,
                            borrow_balance,
                            lending_parameters.sr_borrow_balance,
                            reserve_balance + platform_bonus,
                        )
                    });

            if repaid.resource_address() == deposited_asset {
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.put(
                            repaid,
                            new_total_deposit,
                            new_total_sr_deposit,
                            repaid_asset_total_borrow_balance - decreased_amount,
                            repaid_pool_parameters.sr_borrow_balance - decreased_amount,
                            reserve_balance + platform_bonus,
                        )
                    });
            } else {
                let mut borrowed_pool = self.pools.get(&repaid.resource_address()).unwrap().clone();
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        borrowed_pool.put(
                            repaid,
                            repaid_asset_total_deposit_balance,
                            repaid_pool_parameters.sr_deposit_balance,
                            repaid_asset_total_borrow_balance - decreased_amount,
                            repaid_pool_parameters.sr_borrow_balance - decreased_amount,
                            repaid_pool_parameters.reserve_balance + platform_bonus,
                        )
                    });
            }
            to_return_reward
        }

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

        fn is_approved_by_admins(&mut self) -> bool {
            let singature_count = self.admin_signature_check.len();
            if singature_count < 3 {
                false
            } else {
                true
            }
        }

        pub fn update_pool_parameters(
            &mut self,
            resource_address: ResourceAddress,
            min_liquidable_value: Decimal,
            liquidation_reserve_factor: Decimal,
            liquidation_bonus: Decimal,
            max_liquidation_percent: Decimal,
            max_borrow_percent: Decimal,
            min_collateral_ratio: Decimal,
            pool_reserve: Decimal,
            pool_deposit_limit: Decimal,
        ) {
            info!("update_pool_parameters initiated.");
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_pool_parameters(
                    min_liquidable_value,
                    liquidation_reserve_factor,
                    liquidation_bonus,
                    max_liquidation_percent,
                    max_borrow_percent,
                    min_collateral_ratio,
                    pool_reserve,
                    pool_deposit_limit,
                );

            self.admin_signature_check = HashMap::new();
        }

        fn update_pool_balances(
            &mut self,
            resource_address: ResourceAddress,
            deposit: Decimal,
            sr_deposit: Decimal,
            borrow: Decimal,
            sr_borrow: Decimal,
            reserve: Decimal,
        ) {
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_balances(deposit, sr_deposit, borrow, sr_borrow, reserve);
        }

        fn _get_id_from_proof(&mut self, user_badge: Proof) -> Decimal {
            let manager = ResourceManager::from(user_badge.resource_address());
            let non_fungible_id = user_badge
                .check(manager.address())
                .as_non_fungible()
                .non_fungible_local_id();

            let user_id = match non_fungible_id {
                NonFungibleLocalId::Integer(id) => id.value(),
                _ => panic!("Unexpected NonFungibleLocalId type"),
            };
            let user_id_decimal = Decimal::from(user_id);
            user_id_decimal
        }

        fn _get_non_fungible_id_from_proof(&mut self, user_badge: Proof) -> NonFungibleLocalId {
            let manager = ResourceManager::from(user_badge.resource_address());
            let non_fungible_id = user_badge
                .check(manager.address())
                .as_non_fungible()
                .non_fungible_local_id();

            non_fungible_id
        }

        fn borrowable_amount(
            &mut self,
            total_deposit: Decimal,
            total_borrow: Decimal,
            reserve_balance: Decimal,
            pool_reserve: Decimal,
        ) -> Decimal {
            let available_pool_amount =
                total_deposit - reserve_balance - total_deposit * pool_reserve - total_borrow;
            available_pool_amount
        }
    }
}
