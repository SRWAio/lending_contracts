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
    pub deposits: IndexMap<ResourceAddress, PreciseDecimal>,

    /// All users borrows.
    #[mutable]
    pub borrows: IndexMap<ResourceAddress, PreciseDecimal>,
}
