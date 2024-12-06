use scrypto::prelude::*;
mod calculations;
mod user;

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

    struct LendingProtocol {
        pool_address: Global<Pool>,
        oracle_address: Global<PriceOracle>,
        admin_signature_check: HashMap<NonFungibleLocalId, bool>,
        /// A counter for ID generation
        admin_badge_id_counter: u64,
        admin_badge_address: ResourceAddress,
    }

    impl LendingProtocol {}
}
