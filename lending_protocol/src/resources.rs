use scrypto::prelude::*;

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

pub fn create_protocol_badge(
    owner_rule: AccessRule,
    address_reservation: GlobalAddressReservation,
) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<ProtocolBadge>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
            }
        ))
        .with_address(address_reservation)
        .mint_initial_supply([(
            1u64.into(),
            ProtocolBadge {
                name: "Protocol Badge".to_string(),
            },
        )])
}

pub fn create_user_badge(
    owner_rule: AccessRule,
    address_reservation: GlobalAddressReservation,
    id: u64,
) -> NonFungibleBucket {
    ResourceBuilder::new_integer_non_fungible::<UserBadge>(OwnerRole::None)
        .metadata(metadata!(
            roles {
                metadata_setter => owner_rule.clone();
                metadata_setter_updater => owner_rule.clone();
                metadata_locker => owner_rule.clone();
                metadata_locker_updater => owner_rule;
            }
        ))
        .with_address(address_reservation)
        .mint_initial_supply([(
            id.into(),
            UserBadge {
                name: "User Badge".to_string(),
            },
        )])
}
