use cosmwasm_std::{CanonicalAddr, HumanAddr, Storage};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub static CONFIG_KEY: &[u8] = b"config";
pub static OWNER_KEY: &[u8] = b"owner_key";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Name {
    pub value: String,
    pub owner: CanonicalAddr,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NSOwner {
    pub nameservice_owner: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NamesS {
    pub names_vector: Vec<Name>,
}
pub fn names_store<S: Storage>(storage: &mut S) -> Singleton<S, NamesS> {
    singleton(storage, CONFIG_KEY)
}
pub fn names_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, NamesS> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn nsowner_store<S: Storage>(storage: &mut S) -> Singleton<S, NSOwner> {
    singleton(storage, OWNER_KEY)
}
pub fn nsowner_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, NSOwner> {
    singleton_read(storage, OWNER_KEY)
}
