use scrypto::prelude::*;

/// Data describing the CDP
#[derive(ScryptoSbor, NonFungibleData, Clone, Debug)]
pub struct UserData {
    /// Image to display when exploring Radix transactions
    #[mutable]
    pub image_url: String,

    /// Name of user badge
    #[mutable]
    pub name: String,

    /// Badge minted at
    pub minted_at: i64,

    /// Badge updated at
    #[mutable]
    pub updated_at: i64,

    /// All users deposits.
    #[mutable]
    pub deposits: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub sr_deposits: IndexMap<ResourceAddress, Decimal>,
    /* #[mutable]
    pub deposit_epoch: IndexMap<ResourceAddress, i64>,*/
    /// All users borrows.
    #[mutable]
    pub borrows: IndexMap<ResourceAddress, Decimal>,

    #[mutable]
    pub sr_borrows: IndexMap<ResourceAddress, Decimal>,
    /* #[mutable]
    pub borrow_epoch: IndexMap<ResourceAddress, i64>,*/
}

impl UserData {
    pub fn get_deposits(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.deposits, resource_address)
    }

    pub fn get_sr_deposits(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.sr_deposits, resource_address)
    }

    pub fn get_borrows(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.borrows, resource_address)
    }

    pub fn get_sr_borrows(&self, resource_address: ResourceAddress) -> Decimal {
        Self::get_value(&self.sr_borrows, resource_address)
    }

    fn get_value(map: &IndexMap<ResourceAddress, Decimal>, key: ResourceAddress) -> Decimal {
        map.get(&key).copied().unwrap_or(Decimal::ZERO)
    }

    pub fn update_deposit(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.deposits, res_address, value);
    }

    pub fn update_sr_deposit(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.sr_deposits, res_address, value);
    }

    pub fn update_borrow(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.borrows, res_address, value);
    }

    pub fn update_sr_borrow(&mut self, res_address: ResourceAddress, value: Decimal) {
        Self::update_map(&mut self.sr_borrows, res_address, value);
    }

    pub fn on_deposit(
        &mut self,
        resource_address: ResourceAddress,
        deposit_increase: Decimal,
        sr_deposit_increase: Decimal,
    ) {
        let mut deposit = self.get_deposits(resource_address);
        deposit += deposit_increase;
        let mut sr_deposit = self.get_sr_deposits(resource_address);
        sr_deposit += sr_deposit_increase;
        self.update_deposit(resource_address, deposit);
        self.update_sr_deposit(resource_address, sr_deposit);
    }

    pub fn on_withdraw(
        &mut self,
        resource_address: ResourceAddress,
        deposit_decrease: Decimal,
        sr_deposit_decrease: Decimal,
    ) {
        let mut deposit = self.get_deposits(resource_address);
        deposit -= deposit_decrease;
        let mut sr_deposit = self.get_sr_deposits(resource_address);
        sr_deposit -= sr_deposit_decrease;
        self.update_deposit(resource_address, deposit);
        self.update_sr_deposit(resource_address, sr_deposit);
    }

    pub fn on_borrow(
        &mut self,
        resource_address: ResourceAddress,
        borrow_increase: Decimal,
        sr_borrow_increase: Decimal,
    ) {
        let mut borrow = self.get_borrows(resource_address);
        borrow += borrow_increase;
        let mut sr_borrow = self.get_sr_borrows(resource_address);
        sr_borrow += sr_borrow_increase;
        self.update_borrow(resource_address, borrow);
        self.update_sr_borrow(resource_address, sr_borrow);
    }

    pub fn on_repay(
        &mut self,
        resource_address: ResourceAddress,
        borrow_decrease: Decimal,
        sr_borrow_decrease: Decimal,
    ) {
        let mut borrow = self.get_borrows(resource_address);
        borrow -= borrow_decrease;
        let mut sr_borrow = self.get_sr_borrows(resource_address);
        sr_borrow += sr_borrow_decrease;
        self.update_borrow(resource_address, borrow);
        self.update_sr_borrow(resource_address, sr_borrow);
    }

    fn update_map(
        map: &mut IndexMap<ResourceAddress, Decimal>,
        key: ResourceAddress,
        units: Decimal,
    ) {
        if let Some(entry) = map.get_mut(&key) {
            let new_deposit = *entry + units;
            map.insert(key, new_deposit);
        }
    }
}
