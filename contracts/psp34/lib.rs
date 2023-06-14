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

    use ink::prelude::string::ToString;


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

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum ContractError {
        NotEnoughMoney,
        NotEnoughApple
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
        pub fn change_some_status(&mut self, token_id: Id, number: u32) -> Result<(), PSP34Error> {
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
                .insert(&token_id, &new_status);
            Ok(())
        }

        #[ink(message)]
        pub fn set_lucky_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.change_some_status(token_id.clone(),50)
        }

        #[ink(message)]
        pub fn get_total_status(&self, token_id: Id) -> u32 {
            let original_status = self.get_current_status(token_id.clone()).unwrap_or_else(|| {
                // In case the token_id doesn't exist in the asset_status map, we just return a default status with all fields set to 0.
                Status { hungry: 0, health: 0, happy: 0 }
            });
        
            let new_status = Status {
                hungry: original_status.hungry,
                health: original_status.health,
                happy: original_status.happy,
            };
    
            let total_status = new_status.health as i32 + new_status.happy as i32 - new_status.hungry as i32;
            let result = if total_status > 0 { total_status } else { 0 };
            result as u32
        }

        #[ink(message)]
        pub fn get_condition(&self , token_id: Id) -> u32 {
            let condition = self.get_total_status(token_id);
            // bad condition
            if condition < 100 {
                0
            } 
            // normal condition
            else if condition < 200 {
                1
            } 
            // good condition
            else {
                2
            }
        }

        #[ink(message)]
        pub fn get_condition_url(&self , token_id: Id) -> String {
            let condition = self.get_condition(token_id);
            if condition == 0 {
                self.get_bad_uri()
            } else if condition == 1 {
                self.get_normal_uri()
            } else {
                self.get_good_uri()
            }
        }

        #[ink(message)]
        pub fn get_your_apple(&self, account_id: AccountId) -> u16 {
            self
                .apple_number
                .get(&account_id)
                .unwrap_or_default()
        }

        #[ink(message)]
        pub fn set_your_apple(&mut self, account_id: AccountId, after_apple: u16) -> Result<(), PSP34Error> {
            self
                .apple_number
                .insert(&account_id, &after_apple);
            Ok(())
        }

        #[ink(message)]
        pub fn get_your_money(&self, account_id: AccountId) -> u64 {
            self
                .your_money
                .get(&account_id)
                .unwrap_or_default()
        }

        #[ink(message)]
        pub fn set_your_money(&mut self, account_id: AccountId, after_money: u64)  {
            self.your_money.insert(&account_id, &after_money);
        }

        // #[ink(message)]
        // pub fn stake_your_money(&mut self, account_id: AccountId, stake_money: u64) -> Result<()> {
        // }

        #[ink(message)]
        pub fn get_your_staked_money(&self, account_id: AccountId) -> u64 {

            //　get the current time
            let current_time = Self::env().block_timestamp();
    
            // get your_staked_money
            let staked_money = self
                .your_staked_money
                .get(&account_id)
                .unwrap_or(Default::default());
    
            // get last_staked_time
            let last_staked_time = self
                .last_staked
                .get(&account_id)
                .unwrap_or(Default::default());
            if last_staked_time == 0 || staked_money == 0 {
                return 0
            } else {
                let past_time = current_time - last_staked_time;
                // 60 seconds（60 ※ 1000 miliseconds）
                let past_day = past_time / (10 * 1000) ;
                // Assuming a hypothetical decrease of 5 per unit
                let change_patio = past_day * 1;
                return staked_money + staked_money * change_patio / 100
            }
        }

        #[ink(message)]
        pub fn withdraw_your_money(&mut self, account_id: AccountId) -> Result<(), ContractError> {
            let staked_money = self.get_your_staked_money(account_id);
    
            let current_money = self.get_your_money(account_id.clone());
    
            if staked_money == 0 {
                Err(ContractError::NotEnoughMoney.into())
            } else {
                let result_money = current_money + staked_money;
                // set your_staked_money 0
                self
                .your_staked_money
                .insert(&account_id, &0);
    
                // set your_money 
                self
                    .your_money
                    .insert(&account_id, &result_money);
                Ok(())
            }
        }

        #[ink(message)]
        pub fn minus_your_apple(&mut self, account_id: AccountId) -> Result<(), ContractError> {
        
            // get apple number
            let apple_number = self.get_your_apple(account_id);
    
            if apple_number < 1 {
                Err(ContractError::NotEnoughApple.into())
            } else {
                let after_apple = apple_number - 1;
    
                self
                .apple_number
                .insert(&account_id, &after_apple);
                Ok(())
            }
        }

        #[ink(message)]
        pub fn minus_your_money(&mut self, account_id: AccountId, change_money: u64) -> Result<(), ContractError> {
        
            // get current game money
            let money = self.get_your_money(account_id);
    
            if money < change_money {
                Err(ContractError::NotEnoughMoney.into())
            } else {
                let after_money = money - change_money;
                self.set_your_money(account_id, after_money);
                Ok(())
            }
        }
    
    }
}
