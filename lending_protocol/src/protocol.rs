use crate::calculations::*;
use crate::resources::*;
use crate::user::UserData;
use scrypto::prelude::*;

#[blueprint]
mod lending_protocol {
    use crate::pool_parameters::PoolParameters;

    extern_blueprint! {
    // import the Pool package from the ledger using its package address
    "package_tdx_2_1phyc0v7jt30j98laknzypnw4jqm8celeqv74glrj8lge2n8euvc8nr",
    Pool {
        fn instantiate(
            admin_rule: AccessRule,
            resource_address: ResourceAddress,
        ) -> (Global<Pool>, ComponentAddress);
        fn take(&mut self, amount: Decimal,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,)  -> Bucket;
        fn put(&mut self, bucket: Bucket,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,);
            fn get_pool_balances(&self) -> (Decimal, Decimal, Decimal, Decimal, Decimal);
        }
    }
    extern_blueprint! {
    // import the PriceORacle package from the ledger using its package address
    "package_tdx_2_1ph0hwlqmde3ht29pzy5qehqflvjfrtty4lgyvwhhqp589e0v0qhtke",
    PriceOracle {
        fn get_price(&mut self, res_addr: ResourceAddress) -> Decimal;
        fn get_price_in_xrd(&mut self, res_addr: ResourceAddress) -> Decimal;
        }
    }

    enable_method_auth! {
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
            collect_reserve_balance => restrict_to: [admin];
            insert_pool_component =>  restrict_to: [admin];
            update_pool_parameters => restrict_to: [admin];
            update_balances => restrict_to: [admin];
            update_pool_settings => restrict_to: [admin];
            lock_pool => restrict_to: [admin];
            add_to_blacklist => restrict_to: [admin];
            remove_from_blacklist => restrict_to: [admin];
        }
    }

    struct LendingProtocol {
        pools: KeyValueStore<ResourceAddress, Global<Pool>>,
        protocol_badge: NonFungibleVault,
        admin_rule: AccessRule,
        component_rule: AccessRule,
        protocol_rule: AccessRule,
        oracle_address: Global<PriceOracle>,
        admin_signature_check: HashMap<NonFungibleLocalId, bool>,
        admin_badge_id_counter: u64,
        admin_badge_address: ResourceAddress,
        user_resource_manager: NonFungibleResourceManager,
        pool_parameters: KeyValueStore<ResourceAddress, PoolParameters>,
        ltv_ratios: HashMap<ResourceAddress, Decimal>,
        admin_blacklist: HashSet<NonFungibleLocalId>,
    }

    impl LendingProtocol {
        pub fn instantiate(oracle_address: Global<PriceOracle>) -> NonFungibleBucket {
            // Get address reservation for the lending market component
            let (protocol_component_address_reservation, protocol_component_address) =
                Runtime::allocate_component_address(LendingProtocol::blueprint_id());
            let component_rule = rule!(require(global_caller(protocol_component_address)));

            // Get address reservation for the admin badge resource address
            let (admin_badge_address_reservation, admin_badge_address) =
                Runtime::allocate_non_fungible_address();

            let admin_rule: AccessRule = rule!(require(admin_badge_address));

            let protocol_badge = create_protocol_badge(admin_rule.clone());
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));

            let admin_badges = create_admin_badge(
                protocol_rule.clone(),
                component_rule.clone(),
                admin_rule.clone(),
                admin_badge_address_reservation,
            );

            let user_resource_manager = create_user_resource_manager(
                protocol_rule.clone(),
                component_rule.clone(),
                admin_rule.clone(),
            );

            Self {
                protocol_badge: NonFungibleVault::with_bucket(protocol_badge),
                pools: KeyValueStore::new(),
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                component_rule: component_rule.clone(),
                protocol_rule: protocol_rule,
                admin_signature_check: HashMap::new(),
                admin_badge_address: admin_badges.resource_address(),
                admin_badge_id_counter: 5,
                pool_parameters: KeyValueStore::new(),
                oracle_address,
                ltv_ratios: HashMap::new(),
                admin_blacklist: HashSet::new(),
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

            admin_badges
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
            let admin_rule: AccessRule = rule!(require(admin_badge_address));
            let protocol_rule: AccessRule = rule!(require(protocol_badge.resource_address()));
            let user_resource_manager: NonFungibleResourceManager = user_badge_address.into();
            let admin_resource_manager: NonFungibleResourceManager = admin_badge_address.into();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                protocol_badge.non_fungible_local_ids();
            protocol_badge.authorize_with_non_fungibles(&non_fungible_local_ids, || {
                user_resource_manager.set_mintable(component_rule.clone());
                user_resource_manager.set_burnable(component_rule.clone());
                user_resource_manager.set_updatable_non_fungible_data(component_rule.clone());
                admin_resource_manager.set_mintable(component_rule.clone());
                admin_resource_manager.set_burnable(component_rule.clone());
                admin_resource_manager.set_updatable_non_fungible_data(component_rule.clone())
            });
            Self {
                protocol_badge: NonFungibleVault::with_bucket(protocol_badge),
                pools: KeyValueStore::new(),
                user_resource_manager,
                admin_rule: admin_rule.clone(),
                component_rule: component_rule.clone(),
                admin_signature_check: HashMap::new(),
                protocol_rule,
                admin_badge_address,
                admin_badge_id_counter: 5,
                pool_parameters: KeyValueStore::new(),
                oracle_address,
                ltv_ratios: HashMap::new(),
                admin_blacklist: HashSet::new(),
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
            base: Decimal,
            base_multiplier: Decimal,
            multiplier: Decimal,
            kink: Decimal,
            reserve_factor: Decimal,
            ltv_ratio: Decimal,
        ) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            assert!(
                ltv_ratio >= 0.into() && ltv_ratio <= 1.into(),
                "LTV must be between 0.0 and 1.0."
            );
            assert!(multiplier > 0.into(), "Multiplier must be greater then 0.");
            assert!(
                multiplier > base_multiplier,
                "Multiplier must be greater then Base Multiplier."
            );
            assert!(
                reserve_factor >= 0.into() && reserve_factor <= 1.into(),
                "Reserve Factor must be between 0.0 and 1.0."
            );
            assert!(
                kink >= 0.into() && kink <= 100.into(),
                "Kink must be between 0 and 100."
            );
            if self.ltv_ratios.contains_key(&resource_address) {
                panic!("Pool already exists for this resource address.");
            }
            self.pools.insert(resource_address, pool_component);
            let now = Runtime::current_epoch().number();
            let pool_balances = pool_component.get_pool_balances();
            let data = PoolParameters {
                balances_updated_at: now,
                base,
                base_multiplier,
                multiplier,
                kink,
                reserve_factor,
                ltv_ratio,
                min_collateral_ratio: Decimal::one(),
                max_borrow_percent: dec!("0.1"),
                max_liquidation_percent: dec!("0.5"),
                liquidation_bonus: dec!("0.1"),
                liquidation_reserve_factor: dec!("0.2"),
                deposit_locked: false,
                borrow_locked: false,
                withdraw_locked: false,
                repay_locked: false,
                pool_reserve: dec!("0.2"),
                deposit_limit: dec!("100000"),
                deposit_balance: pool_balances.0,
                sd_balance: pool_balances.1,
                borrow_balance: pool_balances.2,
                sb_balance: pool_balances.3,
                reserve_balance: pool_balances.4,
            };
            self.ltv_ratios.insert(resource_address, ltv_ratio);
            self.pool_parameters.insert(resource_address, data);
            self.admin_signature_check = HashMap::new();
        }

        pub fn create_pool(
            &mut self,
            resource_address: ResourceAddress,
            base: Decimal,
            base_multiplier: Decimal,
            multiplier: Decimal,
            kink: Decimal,
            reserve_factor: Decimal,
            ltv_ratio: Decimal,
        ) -> (Global<Pool>, ComponentAddress) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            assert!(
                ltv_ratio >= 0.into() && ltv_ratio <= 1.into(),
                "LTV must be between 0.0 and 1.0."
            );
            assert!(multiplier > 0.into(), "Multiplier must be greater then 0.");
            assert!(
                multiplier > base_multiplier,
                "Multiplier must be greater than Base Multiplier."
            );
            assert!(
                reserve_factor >= 0.into() && reserve_factor <= 1.into(),
                "Reserve Factor must be between 0.0 and 1.0."
            );
            assert!(
                kink >= 0.into() && kink <= 100.into(),
                "Kink must be between 0 and 100."
            );
            if self.ltv_ratios.contains_key(&resource_address) {
                panic!("Pool already exists for this resource address.");
            }
            let pool_component_address =
                Blueprint::<Pool>::instantiate(self.protocol_rule.clone(), resource_address);
            self.pools
                .insert(resource_address, pool_component_address.0);
            self.ltv_ratios.insert(resource_address, ltv_ratio);
            let now = Runtime::current_epoch().number();

            let data = PoolParameters {
                balances_updated_at: now,
                base,
                base_multiplier,
                multiplier,
                kink,
                reserve_factor,
                ltv_ratio,
                min_collateral_ratio: Decimal::one(),
                max_borrow_percent: dec!("0.1"),
                max_liquidation_percent: dec!("0.5"),
                liquidation_bonus: dec!("0.1"),
                liquidation_reserve_factor: dec!("0.2"),
                deposit_locked: false,
                borrow_locked: false,
                withdraw_locked: false,
                repay_locked: false,
                pool_reserve: dec!("0.2"),
                deposit_limit: dec!("100000"),
                deposit_balance: Decimal::zero(),
                sd_balance: Decimal::zero(),
                borrow_balance: Decimal::zero(),
                sb_balance: Decimal::zero(),
                reserve_balance: Decimal::zero(),
            };

            self.pool_parameters.insert(resource_address, data);
            self.admin_signature_check = HashMap::new();
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
            let mut sd_balance = pool_parameters.sd_balance;

            if pool_deposit_limit > Decimal::ZERO {
                let current_deposit_balance = asset_total_deposit_balance + asset.amount();
                if current_deposit_balance > pool_deposit_limit {
                    panic!("Deposit limit is {} .", pool_deposit_limit);
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

            let sd_interest =
                calculate_s_interest(asset.amount(), asset_total_deposit_balance, sd_balance);
            sd_balance += sd_interest;
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
                name: "SRWA Sandbox Badge v2 - Keep safe!".to_string(),
                key_image_url: "https://demo.srwa.io/images/badge.png".into(),
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
                sd_balance,
                asset_total_borrow_balance,
                pool_parameters.sb_balance,
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
                        sd_balance,
                        asset_total_borrow_balance,
                        pool_parameters.sb_balance,
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
            let mut sd_balance = pool_parameters.sd_balance;

            if pool_deposit_limit > Decimal::ZERO {
                let current_deposit_balance = asset_total_deposit_balance + asset.amount();
                if current_deposit_balance > pool_deposit_limit {
                    panic!("Deposit limit is {} .", pool_deposit_limit);
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

            let sd_interest =
                calculate_s_interest(asset.amount(), asset_total_deposit_balance, sd_balance);
            sd_balance += sd_interest;
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
                sd_balance,
                asset_total_borrow_balance,
                pool_parameters.sb_balance,
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
                        sd_balance,
                        asset_total_borrow_balance,
                        pool_parameters.sb_balance,
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
            let available_liquidity = self.available_liquidity(
                pool_parameters.deposit_balance,
                pool_parameters.borrow_balance,
                pool_parameters.reserve_balance,
                pool_parameters.pool_reserve,
            );
            if available_liquidity < amount {
                panic!("Available liquidity is {}: ", available_liquidity);
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
            let user_deposit_balance = user.get_deposit(resource_address)
                * pool_parameters.deposit_balance
                / pool_parameters.sd_balance;
            if user_deposit_balance < amount {
                panic!(
                    "User does not have enough deposit balance to withdraw. Max withdrawal is: {}",
                    user_deposit_balance
                );
            }
            let total_collateral_and_loan = user.calculate_total_collateral_and_loan(
                &self.pool_parameters,
                &self.ltv_ratios,
                prices.clone(),
            );
            let user_available_collateral =
                total_collateral_and_loan.0 - total_collateral_and_loan.1;
            let withdrawable_amount_in_xrd = user_available_collateral / asset_ltv_ratio;
            let cost_of_asset_in_terms_of_xrd = prices.get(&resource_address).unwrap();
            let withdrawable_amount = withdrawable_amount_in_xrd / *cost_of_asset_in_terms_of_xrd;

            if amount > withdrawable_amount {
                panic!("Max withdrawal amount is {}: ", withdrawable_amount);
            }
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sd_balance = pool_parameters.sd_balance;

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
            let sd_interest = calculate_s_interest(amount, asset_total_deposit_balance, sd_balance);
            asset_total_borrow_balance += interests.0;
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance -= amount;
            sd_balance -= sd_interest;
            self.update_pool_balances(
                resource_address,
                asset_total_deposit_balance,
                sd_balance,
                asset_total_borrow_balance,
                pool_parameters.sb_balance,
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
                            sd_balance,
                            asset_total_borrow_balance,
                            pool_parameters.sb_balance,
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
            let user_badge_resource_address = user_badge.resource_address();
            let manager_address = self.user_resource_manager.address();

            if manager_address != user_badge_resource_address {
                panic!("User does not exist!");
            };
            let pool_parameters = self.pool_parameters.get(&asset_address).unwrap().clone();
            let borrow_locked = pool_parameters.borrow_locked;
            if borrow_locked {
                panic!("Borrowing is locked for now!");
            }

            let available_liquidity = self.available_liquidity(
                pool_parameters.deposit_balance,
                pool_parameters.borrow_balance,
                pool_parameters.reserve_balance,
                pool_parameters.pool_reserve,
            );
            if available_liquidity < amount {
                panic!("Available liquidity amount is {}: ", available_liquidity);
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
            let total_collateral_and_loan = user.calculate_total_collateral_and_loan(
                &self.pool_parameters,
                &self.ltv_ratios,
                prices.clone(),
            );
            let user_available_collateral =
                total_collateral_and_loan.0 - total_collateral_and_loan.1;
            assert!(
                user_available_collateral >= borrow_amount_in_terms_of_xrd,
                "[borrow_asset][POOL] User does not have enough collateral. Requested loan with \
                      value of `{:?}` XRD but only has `{:?}` XRD of available collateral.",
                borrow_amount_in_terms_of_xrd,
                user_available_collateral
            );
            let max_borrow;
            if user_available_collateral
                < pool_parameters.deposit_balance * pool_parameters.max_borrow_percent
            {
                max_borrow = user_available_collateral;
            } else {
                let user_borrow = user.get_borrow(asset_address);
                max_borrow = pool_parameters.max_borrow_percent * pool_parameters.deposit_balance
                    - user_borrow;
            }
            if amount > max_borrow {
                panic!("Max borrow amount is {}: ", max_borrow);
            }
            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sb_balance = pool_parameters.sb_balance;
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
            let sb_interest = calculate_s_interest(amount, asset_total_borrow_balance, sb_balance);
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;
            asset_total_borrow_balance += amount;
            sb_balance += sb_interest;
            self.update_pool_balances(
                asset_address,
                asset_total_deposit_balance,
                pool_parameters.sd_balance,
                asset_total_borrow_balance,
                sb_balance,
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
                            pool_parameters.sd_balance,
                            asset_total_borrow_balance,
                            sb_balance,
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
                panic!("Repaying is locked for now!");
            }

            let mut asset_total_deposit_balance = pool_parameters.deposit_balance;
            let mut asset_total_borrow_balance = pool_parameters.borrow_balance;
            let mut asset_total_reserve_balance = pool_parameters.reserve_balance;
            let mut sb_balance = pool_parameters.sb_balance;
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
            let sb_price = calculate_token_price(asset_total_borrow_balance, sb_balance);
            asset_total_borrow_balance += interests.0;
            let non_fungible_id: NonFungibleLocalId = user_badge
                .check(manager_address)
                .as_non_fungible()
                .non_fungible_local_id();
            let mut user: UserData = self
                .user_resource_manager
                .get_non_fungible_data(&non_fungible_id);
            let user_borrow = user.get_borrow(asset_address);
            let max_repay_amount = user_borrow * sb_price;
            let mut repaid_amount = repaid.amount();
            let mut to_return = Decimal::zero();
            if repaid_amount > max_repay_amount {
                to_return = repaid_amount - max_repay_amount;
                repaid_amount = max_repay_amount;
            }
            let sb_interest =
                calculate_s_interest(repaid_amount, asset_total_borrow_balance, sb_balance);
            asset_total_reserve_balance += interests.1;
            asset_total_deposit_balance += interests.2;

            sb_balance -= sb_interest;

            user.on_repay(asset_address, sb_interest);
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
            asset_total_borrow_balance -= repaid_amount;
            let return_bucket =
                repaid.take_advanced(to_return, WithdrawStrategy::Rounded(RoundingMode::ToZero));
            self.update_pool_balances(
                asset_address,
                asset_total_deposit_balance,
                pool_parameters.sd_balance,
                asset_total_borrow_balance,
                sb_balance,
                asset_total_reserve_balance,
            );
            let mut pool = self.pools.get(&asset_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            self.protocol_badge
                .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                    pool.put(
                        repaid,
                        asset_total_deposit_balance,
                        pool_parameters.sd_balance,
                        asset_total_borrow_balance,
                        sb_balance,
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
            let loan_limit_used =
                user.get_loan_limit_used(&self.pool_parameters, &self.ltv_ratios, prices.clone());

            if loan_limit_used == Decimal::ZERO {
                panic!("No borrow from the user");
            }
            let lending_parameters = self.pool_parameters.get(&deposited_asset).unwrap().clone();

            let min_collateral_ratio = lending_parameters.min_collateral_ratio;

            assert!(
                loan_limit_used > min_collateral_ratio,
                "Liquidation not allowed."
            );
            let deposit_and_borrow_in_xrd =
                user.get_deposit_and_borrow_balance_in_xrd(&self.pool_parameters, &prices);
            let borrow_amount_in_xrd = deposit_and_borrow_in_xrd.1;

            let deposit_amount_in_xrd = deposit_and_borrow_in_xrd.0;

            let asset_max_liquidation_percent = repaid_pool_parameters.max_liquidation_percent;

            let asset_liquidation_bonus = repaid_pool_parameters.liquidation_bonus;

            let asset_liquidation_reserve_factor =
                repaid_pool_parameters.liquidation_reserve_factor;

            let repaid_asset_total_deposit_balance = repaid_pool_parameters.deposit_balance;
            let repaid_asset_total_borrow_balance = repaid_pool_parameters.borrow_balance;
            let repaid_asset_total_sb_balance = repaid_pool_parameters.sb_balance;

            let deposit_balance = lending_parameters.deposit_balance;
            let borrow_balance = lending_parameters.borrow_balance;
            let reserve_balance = lending_parameters.reserve_balance;
            let pool_reserve = lending_parameters.pool_reserve;

            let available_liquidity = self.available_liquidity(
                deposit_balance,
                borrow_balance,
                reserve_balance,
                pool_reserve,
            );

            let sb_price = calculate_token_price(
                repaid_asset_total_borrow_balance,
                repaid_asset_total_sb_balance,
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
                available_liquidity,
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
            let new_total_sd_balance = lending_parameters.sd_balance - reward - platform_bonus;
            let new_repaid_asset_total_borrow_balance =
                repaid_asset_total_borrow_balance - decreased_amount;
            let new_repaid_sb_balance = repaid_asset_total_sb_balance - decreased_amount;
            let to_return_reward =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            reward,
                            new_total_deposit,
                            new_total_sd_balance,
                            borrow_balance,
                            lending_parameters.sb_balance,
                            reserve_balance + platform_bonus,
                        )
                    });
            self.update_pool_balances(
                deposited_asset,
                new_total_deposit,
                new_total_sd_balance,
                borrow_balance,
                lending_parameters.sb_balance,
                reserve_balance + platform_bonus,
            );

            if repaid.resource_address() == deposited_asset {
                self.update_pool_balances(
                    deposited_asset,
                    new_total_deposit,
                    new_total_sd_balance,
                    new_repaid_asset_total_borrow_balance,
                    new_repaid_sb_balance,
                    reserve_balance + platform_bonus,
                );
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.put(
                            repaid,
                            new_total_deposit,
                            new_total_sd_balance,
                            new_repaid_asset_total_borrow_balance,
                            new_repaid_sb_balance,
                            reserve_balance + platform_bonus,
                        )
                    });
            } else {
                let mut borrowed_pool = self.pools.get(&repaid_resource_address).unwrap().clone();

                self.update_pool_balances(
                    repaid_resource_address,
                    repaid_asset_total_deposit_balance,
                    repaid_pool_parameters.sd_balance,
                    new_repaid_asset_total_borrow_balance,
                    new_repaid_sb_balance,
                    repaid_pool_parameters.reserve_balance,
                );
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        borrowed_pool.put(
                            repaid,
                            repaid_asset_total_deposit_balance,
                            repaid_pool_parameters.sd_balance,
                            new_repaid_asset_total_borrow_balance,
                            new_repaid_sb_balance,
                            repaid_pool_parameters.reserve_balance,
                        )
                    });
            }
            to_return_reward
        }

        pub fn collect_reserve_balance(
            &mut self,
            resource_address: ResourceAddress,
            amount: Decimal,
        ) -> Bucket {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            let pool_parameters = self.pool_parameters.get(&resource_address).unwrap().clone();
            let mut reserve_balance = pool_parameters.reserve_balance;

            if amount > reserve_balance {
                panic!("Available reserve balance is: {}", reserve_balance);
            }
            reserve_balance -= amount;
            let mut pool = self.pools.get(&resource_address).unwrap().clone();
            let non_fungible_local_ids: IndexSet<NonFungibleLocalId> =
                self.protocol_badge.non_fungible_local_ids(1);
            let reserve_bucket =
                self.protocol_badge
                    .authorize_with_non_fungibles(&non_fungible_local_ids, || {
                        pool.take(
                            amount,
                            pool_parameters.deposit_balance,
                            pool_parameters.sd_balance,
                            pool_parameters.borrow_balance,
                            pool_parameters.sb_balance,
                            reserve_balance,
                        )
                    });
            self.update_pool_balances(
                resource_address,
                pool_parameters.deposit_balance,
                pool_parameters.sd_balance,
                pool_parameters.borrow_balance,
                pool_parameters.sb_balance,
                reserve_balance,
            );
            self.admin_signature_check = HashMap::new();
            reserve_bucket
        }

        pub fn approve_admin_functions(&mut self, admin_badge: Proof) {
            let manager = ResourceManager::from(admin_badge.resource_address());
            let admin_id = admin_badge
                .check(manager.address())
                .as_non_fungible()
                .non_fungible_local_id();
            if self.admin_blacklist.contains(&admin_id) {
                panic!("Admin is blacklisted!");
            }
            self.admin_signature_check.insert(admin_id, true);
        }

        pub fn add_to_blacklist(&mut self, admin_id: NonFungibleLocalId) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.admin_signature_check = HashMap::new();
            self.admin_blacklist.insert(admin_id);
        }

        pub fn remove_from_blacklist(&mut self, admin_id: NonFungibleLocalId) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.admin_signature_check = HashMap::new();
            self.admin_blacklist.remove(&admin_id);
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
            liquidation_reserve_factor: Decimal,
            liquidation_bonus: Decimal,
            max_liquidation_percent: Decimal,
            max_borrow_percent: Decimal,
            min_collateral_ratio: Decimal,
            pool_reserve: Decimal,
            pool_deposit_limit: Decimal,
        ) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_pool_parameters(
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

        pub fn update_pool_settings(
            &mut self,
            resource_address: ResourceAddress,
            base: Decimal,
            base_multiplier: Decimal,
            multiplier: Decimal,
            kink: Decimal,
            reserve_factor: Decimal,
            ltv_ratio: Decimal,
        ) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_pool_settings(
                    base,
                    base_multiplier,
                    multiplier,
                    kink,
                    reserve_factor,
                    ltv_ratio,
                );
            self.ltv_ratios.insert(resource_address, ltv_ratio);
            self.admin_signature_check = HashMap::new();
        }

        fn update_pool_balances(
            &mut self,
            resource_address: ResourceAddress,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,
        ) {
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_balances(deposit, sd_balance, borrow, sb_balance, reserve);
        }

        pub fn update_balances(
            &mut self,
            resource_address: ResourceAddress,
            deposit: Decimal,
            sd_balance: Decimal,
            borrow: Decimal,
            sb_balance: Decimal,
            reserve: Decimal,
        ) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .update_balances(deposit, sd_balance, borrow, sb_balance, reserve);
            self.admin_signature_check = HashMap::new();
        }

        pub fn lock_pool(
            &mut self,
            resource_address: ResourceAddress,
            deposit_locked: bool,
            borrow_locked: bool,
            withdraw_locked: bool,
            repay_locked: bool,
        ) {
            let is_approved_by_admins = self.is_approved_by_admins();
            if is_approved_by_admins == false {
                panic!("Admin functions must be approved by at least 3 admins")
            }
            self.pool_parameters
                .get_mut(&resource_address)
                .unwrap()
                .lock_pool(deposit_locked, borrow_locked, withdraw_locked, repay_locked);
            self.admin_signature_check = HashMap::new();
        }

        fn available_liquidity(
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
