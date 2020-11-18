use cosmwasm_std::{
    from_slice, to_vec, HumanAddr, ReadonlyStorage, StdError, StdResult, Storage, Uint128,
};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static PAYMENTS: &[u8] = b"payments";
pub static OWNER_CFG: &[u8] = b"owner_cfg";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Money {
    pub amount: Uint128,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Person {
    pub address: HumanAddr, //subscription_status: bool
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Name {
    pub name: String,
}
//If checking the equity of message.sender and name.owner, then we need new definitions

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: HumanAddr, /*instantiator e governance*/
    pub name_service_address: HumanAddr,
}

pub fn owner_cfg_store<S: Storage>(
    storage: &mut S, /*,nsaddress:HumanAddr*/
) -> Singleton<S, Config> {
    singleton(storage, OWNER_CFG)
}
pub fn owner_cfg_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, Config> {
    singleton_read(storage, OWNER_CFG)
}

pub fn payments_store<S: Storage>(
    storage: &mut S,
    key_raw: HumanAddr,
    value_raw: Uint128,
) -> StdResult<()> {
    let key = to_vec(&key_raw)?;
    let value = to_vec(&value_raw)?;
    PrefixedStorage::new(PAYMENTS, storage).set(&key, &value);
    Ok(())
}

pub fn payments_read<S: Storage>(storage: &S, key_raw: HumanAddr) -> StdResult<Uint128> {
    let key = to_vec(&key_raw)?;
    let res = ReadonlyPrefixedStorage::new(PAYMENTS, storage).get(&key);
    match res {
        Some(data) => from_slice(&data),
        None => Err(StdError::generic_err("No payment is found")),
    }
}

pub fn payments_delete<S: Storage>(storage: &mut S, key_raw: HumanAddr) -> StdResult<()> {
    let key = to_vec(&key_raw)?;
    PrefixedStorage::new(PAYMENTS, storage).remove(&key);
    Ok(())
}
