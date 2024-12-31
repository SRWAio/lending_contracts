use crate::calculations::*;
use crate::resources::*;
use crate::user::UserData;
use scrypto::prelude::*;

#[blueprint]
mod lending_protocol {
    use crate::pool_parameters::PoolParameters;

    extern_blueprint! {
    // import the Pool package from the ledger using its package address
    "package_tdx_2_1pkyaukzcnlcw2uvna5qkdgrans3duhsqahnxh86pugpsa4ml255knx",
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
    "package_tdx_2_1p5w9qvrjd0twmwqd5np9kzey688muve8q82ac3ftsq470xc2uxc44k",
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
            take_protocol_badge => restrict_to: [admin];
            create_pool => restrict_to: [admin];
            create_user_and_deposit =>  PUBLIC;
            deposit =>  PUBLIC;
            withdraw =>  PUBLIC;
            borrow =>  PUBLIC;
            repay => PUBLIC;
            insert_pool_component =>  PUBLIC;
            update_pool_parameters => restrict_to: [admin];
        }
    }

    struct LendingProtocol {
        pools: HashMap<ResourceAddress, Global<Pool>>,
        protocol_badge: NonFungibleVault,
        admin_rule: AccessRule,
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
            admin_badge: Proof,
        ) {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let admin_badge_address = admin_badge.resource_address();
            let component_rule = rule!(require(global_caller(protocol_component_address)));
            // Admin will be able to create lending pools, update pool configurations and update operating status
            let admin_rule: AccessRule = rule!(require(admin_badge_address));
            let user_resource_manager: NonFungibleResourceManager = user_badge_address.into();
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));
            let user_resource_manager: NonFungibleResourceManager = user_badge_address.into();

            admin_badge.authorize(|| {
                user_resource_manager.set_updatable_non_fungible_data(component_rule.clone());
                user_resource_manager.set_burnable(component_rule.clone());
                user_resource_manager.set_mintable(component_rule.clone());
            });

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
            /*pool_parameters: PoolParameters,*/
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
                borrow_rate: Decimal::zero(),
                deposit_rate: Decimal::zero(),
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
            let pool_parameters: &PoolParameters =
                self.pool_parameters.get(&resource_address).unwrap();

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
            let sd_reward = calculate_sd_reward(
                asset.amount(),
                asset_total_deposit_balance,
                sr_deposit_balance,
            );
            sr_deposit_balance += sd_reward;
            let mut user_count = match self.user_resource_manager.total_supply() {
                Some(value) => value,
                None => Decimal::zero(),
            };
            user_count += Decimal::one();
            let user_id_converted: u64 = user_count.try_into().unwrap();
            let user_id = NonFungibleLocalId::Integer(user_id_converted.into());
            let now = Clock::current_time(TimePrecision::Second).seconds_since_unix_epoch;
            let mut deposits = IndexMap::new();
            deposits.insert(resource_address, asset_amount);
            let mut sr_deposits = IndexMap::new();
            sr_deposits.insert(resource_address, sd_reward);
            let mut deposit_epoch = IndexMap::new();
            deposit_epoch.insert(resource_address, now);

            let data = UserData {
                name: "User Badge".into(),
                image_url: "https://demo.srwa.io/images/badge.png".into(),
                deposits,
                sr_deposits,
                borrows: IndexMap::new(),
                sr_borrows: IndexMap::new(),
                deposit_epoch,
                borrow_epoch: IndexMap::new(),
                minted_at: now,
                updated_at: now,
            };
            let user = self.user_resource_manager.mint_non_fungible(&user_id, data);
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2 + asset_amount;
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
            let pool_parameters: &PoolParameters =
                self.pool_parameters.get(&resource_address).unwrap();

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
            let sd_reward = calculate_sd_reward(
                asset.amount(),
                asset_total_deposit_balance,
                sr_deposit_balance,
            );
            sr_deposit_balance += sd_reward;
            let manager_address = self.user_resource_manager.address();
            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2 + asset_amount;
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
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
            // Commented out for now, will be updated later
            // sr_tokens: Bucket,
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

            let borrowable_amount = self.borrowable_amount(
                pool_parameters.deposit_balance,
                pool_parameters.borrow_balance,
                pool_parameters.reserve_balance,
                pool_parameters.pool_reserve,
            );
            if borrowable_amount < amount {
                panic!("Max withdrawal amount (1) is {}: ", borrowable_amount);
            }

            //TO DO: Go through all users resources and find ltv ratios
            let asset_ltv_ratio = pool_parameters.ltv_ratio;
            let mut prices = HashMap::new();
            for (&res_address, &_ratio) in &self.ltv_ratios {
                let mut price_in_xrd = Decimal::ONE;
                if res_address != XRD {
                    price_in_xrd = self.oracle_address.get_price_in_xrd(res_address);
                }
                prices.insert(res_address, price_in_xrd);
            }

            let user_available_collateral = Decimal::one();
            let withdrawable_amount_in_xrd = user_available_collateral / asset_ltv_ratio;
            let cost_of_asset_in_terms_of_xrd = prices.get(&resource_address).unwrap();
            let withdrawable_amount = withdrawable_amount_in_xrd / *cost_of_asset_in_terms_of_xrd;

            if amount > withdrawable_amount {
                panic!("Max withrawal amount is {}: ", withdrawable_amount);
            }
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;

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
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2 - amount;

            let amount_to_decrease = amount - Decimal::one();
            //TO DO: Update users NFT
            //user.on_withdraw(asset_address, amount_to_decrease);
            let amount_to_take = Decimal::one();
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let withdrawn_asset =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            amount_to_take,
                            asset_total_deposit_balance,
                            pool_parameters.sr_deposit_balance,
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
            // Check if user has enough collateral
            let user_available_collateral =
                Decimal::one()/*calculations::calculate_available_collateral(&user, &asset_ltv_ratios, &prices)*/;
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
            asset_total_borrow_balance += interests.0 + amount;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;
            Decimal::one()/*calculations::calculate_r_deposit(r_borrow, reserve_factor, utilisation)*/;
            /*let mut user = self.lending_address.get_user(user_id);
            user.update_balances(asset_address, r_borrow, r_deposit);
            user.on_borrow(asset_address, amount);*/
            let amount_to_take = Decimal::one();
            let reserve_amount = Decimal::one();
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let mut pool = self.pools.get(&asset_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let borrowed_asset =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            amount_to_take,
                            asset_total_deposit_balance,
                            pool_parameters.sr_deposit_balance,
                            asset_total_borrow_balance,
                            pool_parameters.sr_borrow_balance,
                            asset_total_reserve_balance,
                        )
                    });

            borrowed_asset
        }

        pub fn repay(&mut self, repaid: Bucket, user_badge: Proof) /*-> Bucket*/
        {
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
            asset_total_borrow_balance += interests.0 - repaid.amount();
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;
            let non_fungible_id = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            //let to_return
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
                        pool_parameters.sr_borrow_balance,
                        asset_total_reserve_balance,
                    )
                });

            //to_return_amount
        }
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
