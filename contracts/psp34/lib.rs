#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod my_psp34_mintable {
    use openbrush::{
        contracts::psp34::extensions::mintable::*,
        contracts::psp34::extensions::enumerable::*,
        traits::Storage,
    };
    use openbrush::traits::String;

    use openbrush::storage::Mapping;


    #[derive(scale::Encode, scale::Decode, Debug, Clone, PartialEq, Default)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Status {
        pub hungry: u32,
        pub health: u32,
        pub happy: u32,
    }

    #[derive(Default, Storage)]
    #[ink(storage)]
    pub struct Contract {
        // #[storage_field]
        // psp34: psp34::Data,

        #[storage_field]
        psp34: psp34::Data<enumerable::Balances>,

        // pub asset_status: Mapping<Id, Status>,
        pub asset_status: Mapping<Id, Status>,

        // this is three pattern uri

        pub normal_uri: String,
        pub good_uri: String,
        pub bad_uri: String,

        // for random
        pub salt: u64,

        // last eaten time
        pub last_eaten: Mapping<Id, u64>,

        // last daily bonus time
        pub last_bonus: Mapping<AccountId, u64>,

        // last staked time
        pub last_staked: Mapping<AccountId, u64>,

        // apple number the account has
        pub apple_number: Mapping<AccountId, u16>,

        // game money the account has
        pub your_money: Mapping<AccountId, u64>,

        // staked game noney the account has
        pub your_staked_money: Mapping<AccountId, u64>,
    }

    impl PSP34 for Contract {}

    impl PSP34Mintable for Contract {}

    impl PSP34Enumerable for Contract {}

    impl Contract {
        /// The constructor
        #[ink(constructor)]
        pub fn new() -> Self {
            Self::default()
        }
        // normal
        #[ink(message)]
        pub fn set_normal_uri(&mut self, normal_uri:String) -> Result<(), String>{
            self.normal_uri = normal_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_normal_uri(&self) -> String {
            self.normal_uri.clone()
        }

        // good
        #[ink(message)]
        pub fn set_good_uri(&mut self, good_uri:String) -> Result<(), String>{
            self.good_uri = good_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_good_uri(&self) -> String {
            self.good_uri.clone()
        }

        // bad
        #[ink(message)]
        pub fn set_bad_uri(&mut self, bad_uri:String) -> Result<(), String>{
            self.bad_uri = bad_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_bad_uri(&self) -> String {
            self.bad_uri.clone()
        }

        #[ink(message)]
        pub fn ensure_exists_and_get_owner(&self, id: Id) -> Result<AccountId, PSP34Error> {
            let token_owner = self
                .psp34
                .owner_of(id.clone())
                .ok_or(PSP34Error::TokenNotExists)?;
            Ok(token_owner)
        }

        #[ink(message)]
        pub fn set_status (
            &mut self,
            token_id: Id, 
            hungry: u32,
            health: u32,
            happy: u32
        ) -> Result<(), PSP34Error>{ 
            self.ensure_exists_and_get_owner(token_id.clone())?;
            self.asset_status
                .insert(
                    &token_id,
                    &Status {
                        hungry,
                        health,
                        happy,
                    },
                );
            Ok(())
        }

        #[ink(message)]
        pub fn set_full_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.set_status(token_id, 0, 100, 100)
        }

        #[ink(message)]
        pub fn set_death_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.set_status(token_id, 80, 0, 0)
        }

        #[ink(message)]
        pub fn get_status(&self, token_id: Id) -> Option<Status> {
            self.asset_status.get(&token_id)
        }

        #[ink(message)]
        pub fn get_current_status(&self, token_id: Id) -> Option<Status> {

            //　get the current time
            let current_time = Self::env().block_timestamp();
    
             // get the last eaten time
             let last_checked_time = self
                .last_eaten
                .get(&token_id)
                .unwrap_or(Default::default());
            if last_checked_time == 0 {
                return Some(Status {
                    hungry: 0,
                    health: 0,
                    happy: 0,
                });
            } else {
            
                let past_time = current_time - last_checked_time;
    
                // 60 seconds（60 ※ 1000 miliseconds）
                let past_day = past_time / (60 * 1000) ;
                // Assuming a hypothetical decrease of 5 per unit
                let change_status = past_day * 5;
    
                let original_status = self.get_status(token_id.clone()).unwrap_or_else(|| {
                    // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
                    Status { hungry: 0, health: 0, happy: 0 }
                });
    
                let new_hungy_status = original_status.hungry + (change_status as u32);
                let new_health_status = original_status.health.saturating_sub(change_status as u32);
                let new_happy_status = original_status.happy.saturating_sub(change_status as u32);
    
                return Some(Status {
                    hungry: new_hungy_status,
                    health: new_health_status,
                    happy: new_happy_status,
                });
            }
        }

        #[ink(message)]
        pub fn change_some_status(&mut self, token_id: Id, number: u32) -> Result<()> {
            let original_status = self.get_current_status(token_id.clone()).unwrap_or_else(|| {
                // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
                Status { hungry: 0, health: 0, happy: 0 }
            });
    
            let hungry_status: u32;
            if original_status.hungry > number {
                hungry_status = original_status.hungry - number;
            } else {
                hungry_status = 0;
            }
        
            let new_status = Status {
                hungry: hungry_status,
                health: original_status.health + number,
                happy: original_status.happy + number,
            };
        
            self
                .asset_status
                .insert(token_id, &new_status);
            Ok(())
        }
    }
}
