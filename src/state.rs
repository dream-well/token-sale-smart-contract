use cosmwasm_std::{HumanAddr, ReadonlyStorage, StdResult, Storage, Uint128};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};
use schemars::JsonSchema;
use secret_toolkit::serialization::{Bincode2, Serde};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// === CONSTANTS ===
pub const CONFIG_KEY: &[u8] = b"config";
pub const USERS_PREFIX: &[u8] = b"users";

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct SecretContract {
    pub address: HumanAddr,
    pub contract_hash: String,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct User {
    pub total_investment: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub accepted_token: SecretContract,
    pub admin: HumanAddr,
    pub offered_token: SecretContract,
    pub sale_end_time: u64,
    pub viewing_key: String,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

// === Users Storage ===
pub struct UsersReadonlyStorage<'a, S: Storage> {
    storage: ReadonlyPrefixedStorage<'a, S>,
}
impl<'a, S: Storage> UsersReadonlyStorage<'a, S> {
    pub fn from_storage(storage: &'a S) -> Self {
        Self {
            storage: ReadonlyPrefixedStorage::new(USERS_PREFIX, storage),
        }
    }

    pub fn get_user(&self, key: &[u8]) -> Option<User> {
        self.as_readonly().get(key)
    }

    // private

    fn as_readonly(&self) -> ReadonlyUsersStorageImpl<ReadonlyPrefixedStorage<S>> {
        ReadonlyUsersStorageImpl(&self.storage)
    }
}

pub struct UsersStorage<'a, S: Storage> {
    storage: PrefixedStorage<'a, S>,
}
impl<'a, S: Storage> UsersStorage<'a, S> {
    pub fn from_storage(storage: &'a mut S) -> Self {
        Self {
            storage: PrefixedStorage::new(USERS_PREFIX, storage),
        }
    }

    pub fn get_user(&self, key: &[u8]) -> Option<User> {
        self.as_readonly().get(key)
    }

    pub fn set_user(&mut self, key: &[u8], value: User) {
        save(&mut self.storage, &key, &value).ok();
    }

    // private

    fn as_readonly(&self) -> ReadonlyUsersStorageImpl<PrefixedStorage<S>> {
        ReadonlyUsersStorageImpl(&self.storage)
    }
}

struct ReadonlyUsersStorageImpl<'a, S: ReadonlyStorage>(&'a S);
impl<'a, S: ReadonlyStorage> ReadonlyUsersStorageImpl<'a, S> {
    pub fn get(&self, key: &[u8]) -> Option<User> {
        let user: Option<User> = may_load(self.0, &key).ok().unwrap();
        user
    }
}

// === PRIVATE ===
fn may_load<T: DeserializeOwned, S: ReadonlyStorage>(
    storage: &S,
    key: &[u8],
) -> StdResult<Option<T>> {
    match storage.get(key) {
        Some(value) => Bincode2::deserialize(&value).map(Some),
        None => Ok(None),
    }
}

fn save<T: Serialize, S: Storage>(storage: &mut S, key: &[u8], value: &T) -> StdResult<()> {
    storage.set(key, &Bincode2::serialize(value)?);
    Ok(())
}
