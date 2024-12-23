use scrypto::prelude::*;

use crate::user::UserData;

#[derive(ScryptoSbor, NonFungibleData)]
pub struct AdminBadge {
    pub name: String,
}
#[derive(ScryptoSbor, NonFungibleData)]
pub struct ProtocolBadge {
    pub name: String,
}
#[derive(ScryptoSbor, NonFungibleData)]
pub struct UserBadge {
    pub name: String,
}

pub fn create_admin_badge(
    owner_rule: AccessRule,
    address_reservation: GlobalAddressReservation,
) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
            },
            init {
                "name" => "Admin Badge", locked;
            }
        ))
        .with_address(address_reservation)
        .mint_initial_supply([
            (
                1u64.into(),
                AdminBadge {
                    name: "Admin 1".to_string(),
                },
            ),
            (
                2u64.into(),
                AdminBadge {
                    name: "Admin 2".to_string(),
                },
            ),
            (
                3u64.into(),
                AdminBadge {
                    name: "Admin 3".to_string(),
                },
            ),
            (
                4u64.into(),
                AdminBadge {
                    name: "Admin 4".to_string(),
                },
            ),
            (
                5u64.into(),
                AdminBadge {
                    name: "Admin 5".to_string(),
                },
            ),
        ])
}

pub fn create_protocol_badge(owner_rule: AccessRule) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<ProtocolBadge>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
            },
            init {
                "name" => "Protocol Badge", locked;
            }
        ))
        .mint_initial_supply([(
            1u64.into(),
            ProtocolBadge {
                name: "Protocol Badge".to_string(),
            },
        )])
}

pub fn create_user_resource_manager(
    owner_rule: AccessRule,
    component_rule: AccessRule,
) -> NonFungibleResourceManager {
    ResourceBuilder::new_integer_non_fungible::<UserData>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule.clone();
            }
        ))
        .mint_roles(mint_roles! {
          minter => component_rule.clone();
          minter_updater => owner_rule.clone();
        })
        .burn_roles(burn_roles! {
          burner => component_rule.clone();
          burner_updater => owner_rule.clone();
        })
        .non_fungible_data_update_roles(non_fungible_data_update_roles! {
          non_fungible_data_updater => component_rule.clone();
          non_fungible_data_updater_updater => owner_rule.clone();
        })
        .create_with_no_initial_supply()
        .into()
}
