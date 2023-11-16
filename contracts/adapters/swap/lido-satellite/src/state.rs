use cosmwasm_std::Addr;
use cw_storage_plus::Item;

pub const ENTRY_POINT_CONTRACT_ADDRESS: Item<Addr> = Item::new("entry_point_contract_address");
pub const LIDO_SATELLITE_CONTRACT_ADDRESS: Item<Addr> =
    Item::new("lido_satellite_contract_address");

pub const CANONICAL_DENOM: Item<String> = Item::new("canonical_denom");
pub const BRIDGED_DENOM: Item<String> = Item::new("bridged_denom");
