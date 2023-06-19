#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod my_psp34_mintable {
    use openbrush::{
        contracts::{
            ownable::*,
            psp22::extensions::{
                mintable::*,
            },
            psp34::extensions::{
                enumerable::*,
                mintable::*,
            },
        },
        traits::{
            Storage,
            String,
        },
        storage::Mapping,
        modifiers,
    };

    use my_psp22_mintable::{ Psp22ContractRef};
    
    use ink::env::hash;

    use ink::prelude::{
        string::ToString,
        vec::Vec,
    };

    use core::{time::Duration};

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
        PSP22Error,
        PSP34Error,
        NotEnoughMoney,
        NotEnoughApple,
        InvalidAccountId,
        TimeHasNotPassed
    }

    impl From<PSP22Error> for ContractError {
        fn from(_: PSP22Error) -> Self {
            Self::PSP22Error
        }
    }

    impl From<PSP34Error> for ContractError {
        fn from(_: PSP34Error) -> Self {
            Self::PSP34Error
        }
    }
    

    #[derive(Default, Storage)]
    #[ink(storage)]
    pub struct Contract {
        // #[storage_field]
        // psp34: psp34::Data,

        #[storage_field]
        psp34: psp34::Data<enumerable::Balances>,

        #[storage_field]
        ownable: ownable::Data,

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

        #[ink(constructor)]
        pub fn new_with_owner(owner: AccountId) -> Self {
            let mut instance = Self::default();
            instance.set_owner(owner);
            instance
        }

        pub fn set_owner(&mut self, owner: AccountId) {
            self.ownable.owner = owner;
        }

        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_default(&mut self, account_id: AccountId) -> Result<(), PSP34Error> {
            self.set_bad_uri(String::from("ipfs://QmV1VxGsrM4MLNn1qwR9Hmu5DGFfWjzHmhHFXpTT2fevMQ/"))?;
            self.set_normal_uri(String::from("ipfs://QmTBf9GJLiw97v84Q7aEPPFHUXdyqXWC6AUp97VnLFZtWr/"))?;
            self.set_good_uri(String::from("ipfs://QmQUxL1RSWbZAWhQfWnJJrMVZsPm4Stc5C64kRuSnXe56Q/"))?;
            self.set_your_apple(account_id, 10);
            self.set_your_money(account_id, 500);
            Ok(())
        }
    
        // normal
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_normal_uri(&mut self, normal_uri:String) -> Result<(), PSP34Error> {
            self.normal_uri = normal_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_normal_uri(&self) -> String {
            self.normal_uri.clone()
        }

        // good
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_good_uri(&mut self, good_uri:String) -> Result<(), PSP34Error> {
            self.good_uri = good_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_good_uri(&self) -> String {
            self.good_uri.clone()
        }

        // bad
        #[ink(message)]
        #[modifiers(only_owner)]
        pub fn set_bad_uri(&mut self, bad_uri:String) -> Result<(), PSP34Error>{
            self.bad_uri = bad_uri;
            Ok(())
        }

        #[ink(message)]
        pub fn get_bad_uri(&self) -> String {
            self.bad_uri.clone()
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
            self.asset_status.insert(&token_id,&Status {hungry,health,happy});
            Ok(())
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
            let last_checked_time = self.last_eaten.get(&token_id).unwrap_or(Default::default());

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
            self.ensure_exists_and_get_owner(token_id.clone())?;
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
        
            self.asset_status.insert(&token_id, &new_status);
            Ok(())
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
        pub fn eat_an_apple(&mut self, token_id: Id, account_id: AccountId) -> Result<(),ContractError> {

            // get last eaten time
            let last_eaten = self.get_last_eaten(token_id.clone());
            // get whether time passed
            let has_passed = self.five_minutes_has_passed(last_eaten);

            if has_passed ==false {
                Err(ContractError::TimeHasNotPassed.into())
            } else {
                // get current time 
                let current_time = Self::env().block_timestamp();
                //  set last eaten time
                self.set_last_eaten(token_id.clone(), current_time);
                //  minus apple
                self.subtract_your_apple(account_id)?;

                // branching by pseudo random
                let random = self.get_pseudo_random(100);
                if random < 25 {
                    self.change_some_status(token_id, 30)?;
                    Ok(())
                } else if random < 50 {
                    self.set_full_status(token_id)?;
                    Ok(())
                } else if random < 75 {
                    self.set_lucky_status(token_id)?;
                    Ok(())
                } else {
                    self.set_death_status(token_id)?;
                    Ok(())
                } 
            }
        }
        #[ink(message)]
        pub fn token_uri(&self , token_id: Id) -> String {
            let id_string:ink::prelude::string::String = match token_id.clone() {
                Id::U8(u8) => {
                    let tmp: u8 = u8;
                    tmp.to_string()
                }
                Id::U16(u16) => {
                    let tmp: u16 = u16;
                    tmp.to_string()
                }
                Id::U32(u32) => {
                    let tmp: u32 = u32;
                    tmp.to_string()
                }
                Id::U64(u64) => {
                    let tmp: u64 = u64;
                    tmp.to_string()
                }
                Id::U128(u128) => {
                    let tmp: u128 = u128;
                    tmp.to_string()
                }
                // _ => "0".to_string()
                Id::Bytes(value) => ink::prelude::string::String::from_utf8(value.clone()).unwrap(),
            };
    
            let base_uri:String = self.get_condition_url(token_id.clone());
            let tmp_uri: ink::prelude::string::String = ink::prelude::string::String::from_utf8(base_uri).unwrap();
            let uri:ink::prelude::string::String = tmp_uri + &id_string;
    
            uri.into_bytes()
        }

        #[ink(message)]
        pub fn get_your_apple(&self, account_id: AccountId) -> u16 {
            self.apple_number.get(&account_id).unwrap_or_default()
        }

        #[ink(message)]
        pub fn get_your_money(&self, account_id: AccountId) -> u64 {
            self.your_money.get(&account_id).unwrap_or_default()
        }

        #[ink(message)]
        pub fn stake_your_money(&mut self, account_id: AccountId, stake_money: u64) -> Result<(), ContractError> {

            //　get the current time
            let current_time = Self::env().block_timestamp();

            //　get the current money
            let current_money = self.get_your_money(account_id.clone());

            //　get the current staked money
            let current_staked_money = self.get_your_staked_money(account_id.clone());

            if current_money == 0 || current_money < stake_money {
                Err(ContractError::NotEnoughMoney.into())
            } else {
                let after_money = current_money - stake_money;

                let after_staked_money = current_staked_money + stake_money;
                // set your_money 0
                self.your_money.insert(&account_id, &after_money);

                // set your_staked_money
                self.your_staked_money.insert(&account_id, &after_staked_money);

                // set last_staked
                self.last_staked.insert(&account_id, &current_time);
                Ok(())
            }
        }


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
        pub fn buy_an_apple(&mut self, account_id: AccountId) -> Result<(), ContractError>{

            // the apple price is 20
            self.subtract_your_money(account_id, 20)?;
    
            // add 1
            let after_apple = self.get_your_apple(account_id) + 1;
            self.apple_number.insert(&account_id, &after_apple);
            Ok(())
        }

        #[ink(message)]
        pub fn get_last_eaten(&self, token_id: Id) -> u64 {
            self.last_eaten.get(&token_id).unwrap_or(Default::default())
        }
        
        #[ink(message)]
        pub fn get_last_bonus(&self, account_id: AccountId) -> u64 {
            self.last_bonus.get(&account_id).unwrap_or(Default::default())
        } 

        #[ink(message)]
        pub fn daily_bonus(&mut self, account_id: AccountId) -> Result<(), ContractError> {

            // Get the time when the last bonus was obtained. In case of error, return 0 
            let last_bonus = self.get_last_bonus(account_id);
            // Function of whether a predetermined amount of time has elapsed.
            let has_passed = self.five_minutes_has_passed(last_bonus);

            //  If the allotted time has not elapsed
            if has_passed ==false {
                Err(ContractError::TimeHasNotPassed.into())
            } else {
            //　Get the current time
            let current_time = Self::env().block_timestamp();
            //  Put current time in last_bonus
            self.set_last_bonus(account_id, current_time);

            let after_money = self.get_your_money(account_id) + 100;
            self.set_your_money(account_id, after_money);

            Ok(())
            }
        }

        #[ink(message)]
        pub fn call_psp22_transfer(&mut self, target_account_id:AccountId, to: AccountId, value: Balance, data: Vec<u8>)  -> Result<(), PSP22Error> {
            let mut interface: Psp22ContractRef = ink::env::call::FromAccountId::from_account_id(target_account_id);
            let from = Self::env().caller();
            interface.transfer_from_contract(from, to, value, data)?;
            Ok(())
        }

        // internal function

        pub fn is_account_id(&self, account_id: AccountId) -> bool {
            let caller = Self::env().caller();
            if caller == account_id {
                true
            } else {
                false
            }
        }

        pub fn is_nft_owner(&self, token_id: Id) -> bool {
            let token_owner = self.owner_of(token_id.clone()).unwrap();
    
            if token_owner == Self::env().caller() {
                true
            } else {
                false
            }
        }

        pub fn set_last_bonus(&mut self, account_id: AccountId, current_time: u64) {
            self.last_bonus.insert(&account_id, &current_time);
        }

        pub fn set_last_eaten(&mut self, token_id: Id, current_time: u64) {
            self.last_eaten.insert(&token_id, &current_time);
        }

        pub fn plus_your_money(&mut self, account_id: AccountId, change_money: u64) {
        
            // get current game money
            let money = self.get_your_money(account_id);
    
            let after_money = money + change_money;
            self.set_your_money(account_id, after_money);
        }

        pub fn subtract_your_money(&mut self, account_id: AccountId, change_money: u64) -> Result<(), ContractError> {
        
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

        pub fn subtract_your_apple(&mut self, account_id: AccountId) -> Result<(), ContractError> {
        
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

        pub fn set_your_money(&mut self, account_id: AccountId, after_money: u64)  {
            self.your_money.insert(&account_id, &after_money);
        }

        pub fn set_your_apple(&mut self, account_id: AccountId, after_apple: u16) {
            self.apple_number.insert(&account_id, &after_apple);
        }

        pub fn set_lucky_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.change_some_status(token_id.clone(),50)?;
            Ok(())
        }

        pub fn set_full_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.set_status(token_id, 0, 100, 100)?;
            Ok(())
        }

        pub fn set_death_status(&mut self, token_id: Id) -> Result<(), PSP34Error> {
            self.set_status(token_id, 80, 0, 0)?;
            Ok(())
        }

        pub fn ensure_exists_and_get_owner(&self, id: Id) -> Result<AccountId, PSP34Error> {
            let token_owner = self
                .psp34
                .owner_of(id.clone())
                .ok_or(PSP34Error::TokenNotExists)?;
            Ok(token_owner)
        }

        pub fn has_passed(&self, check_time :u64, last_time :u64) -> bool{
            let current_time = Self::env().block_timestamp();
            let time_since_last_time = current_time - last_time;
            let duration_time = Duration::from_secs(check_time);
            if Duration::from_millis(time_since_last_time) > duration_time {
                true
            } else {
                false
            }
        }
        
        pub fn five_minutes_has_passed(&self, last_time :u64) -> bool{
            self.has_passed(60,last_time)
        }

        pub fn one_day_has_passed(&self, last_time :u64) -> bool{
            self.has_passed(60 * 60 * 24 ,last_time)
        }

        pub fn get_pseudo_random(&mut self, max_value: u8) -> u8 {
            let seed = Self::env().block_timestamp();
            let mut input: Vec<u8> = Vec::new();
            input.extend_from_slice(&seed.to_be_bytes());
            input.extend_from_slice(&self.salt.to_be_bytes());
            let mut output = <hash::Keccak256 as hash::HashOutput>::Type::default();
            ink::env::hash_bytes::<hash::Keccak256>(&input, &mut output);
            self.salt += 1;
            let number = output[0] % (max_value + 1);
            number
        }
    
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink::env::test;

        fn default_accounts() -> test::DefaultAccounts<ink::env::DefaultEnvironment> {
            test::default_accounts::<ink::env::DefaultEnvironment>()
        }

        fn set_caller(sender: AccountId) {
            ink::env::test::set_caller::<Environment>(sender);
        }
        fn set_block_timestamp(timestamp: u64) {
            ink::env::test::set_block_timestamp::<Environment>(timestamp);
        }

        #[ink::test]
        fn default_apple_value() {
            let contract = Contract::new();
            let account = AccountId::from([0x0; 32]);
            assert_eq!(contract.get_your_apple(account), 0);
        }

        #[ink::test]
        fn set_and_get_apple() {
            let mut contract = Contract::new();
            let account = AccountId::from([0x1; 32]);
            contract.set_your_apple(account, 10);
            assert_eq!(contract.get_your_apple(account), 10);
        }
        #[ink::test]
        fn set_default_works() {
            let accounts = default_accounts();
            let mut contract = Contract::new_with_owner(accounts.alice);

            set_caller(accounts.alice);
            assert!(contract.set_default(accounts.alice.clone()).is_ok());

            assert_eq!(contract.get_bad_uri(), String::from("ipfs://QmV1VxGsrM4MLNn1qwR9Hmu5DGFfWjzHmhHFXpTT2fevMQ/"));
            assert_eq!(contract.get_normal_uri(), String::from("ipfs://QmTBf9GJLiw97v84Q7aEPPFHUXdyqXWC6AUp97VnLFZtWr/"));
            assert_eq!(contract.get_good_uri(), String::from("ipfs://QmQUxL1RSWbZAWhQfWnJJrMVZsPm4Stc5C64kRuSnXe56Q/"));
            assert_eq!(contract.get_your_apple(accounts.alice.clone()), 10);
            assert_eq!(contract.get_your_money(accounts.alice.clone()), 500);
        }
        
        #[ink::test]
        fn get_current_status_works() {
            let accounts = default_accounts();
            set_caller(accounts.alice);
            let mut contract = Contract::new_with_owner(accounts.alice);
            let token_id: Id = Id::U32(1);
             // mint a new token
            assert!(contract.mint(accounts.alice, token_id.clone()).is_ok());

            contract.set_status(token_id.clone(), 100, 100, 100).unwrap();
            let initial_status = contract.get_status(token_id.clone()).unwrap();
            assert_eq!(initial_status, Status { hungry: 100, health: 100, happy: 100 });

            // get current time. but return 0 in test environment
            // let current_time = ink::env::block_timestamp::<ink::env::DefaultEnvironment>().into();
        
            // assume already eaten an apple at 1 second
            contract.set_last_eaten(token_id.clone(), 1 * 1000); // 1 second

            // Let's simulate the passage of time
            set_block_timestamp(61 * 1000); // 61 seconds
            let status_after_time = contract.get_current_status(token_id.clone()).unwrap();
            
            // // We need to manually calculate the expected new statuses because they are time-dependent
            let expected_status = Status {
                hungry: 105, // 100 + 5 (1 minute passed, so status increases by 5)
                health: 95, // 100 - 5
                happy: 95, // 100 - 5
            };
            assert_eq!(status_after_time, expected_status);

            let total_status = contract.get_total_status(token_id.clone());

            assert_eq!(total_status, 85); // 95 + 95 - 105

            set_block_timestamp(6000 * 1000); // 6000 seconds (100 minutes)

            let status_after_many_time_passed = contract.get_current_status(token_id.clone()).unwrap();

            let expected_status_many_time_passed = Status {
                hungry: 595, // 100 + 5 * 99 (100 minute passed, so status increases by 5)
                health: 0, // 100 - 5 * 99 , but not less than 0
                happy: 0, // 100 - 5 * 99 , but not less than 0
            };
            assert_eq!(status_after_many_time_passed, expected_status_many_time_passed);

            let total_status_many_time_passed = contract.get_total_status(token_id.clone());

            assert_eq!(total_status_many_time_passed, 0); // 0 + 0 - 595, but not less than 0

        }

        #[ink::test]
        fn buy_an_apple_works() {
            let mut contract = Contract::default();
            let accounts = test::default_accounts::<Environment>();
            
            // 事前条件：まずアカウントに十分なお金を持たせます。
            contract.set_your_money(accounts.alice, 50);
    
            // アクション：アカウントがりんごを購入します。
            assert!(contract.buy_an_apple(accounts.alice).is_ok());
    
            // アサーション：りんごを1つ持っていることを確認します。
            assert_eq!(contract.get_your_apple(accounts.alice), 1);
        }

        #[ink::test]
        fn buy_an_apple_fails_without_enough_money() {
            let mut contract = Contract::default();
            let accounts = test::default_accounts::<Environment>();

            // 事前条件：アカウントはお金を持っていません。

            // アクション：アカウントがりんごを購入しようとします。
            // アサーション：購入は失敗します。
            assert!(contract.buy_an_apple(accounts.alice).is_err());
        }
        #[ink::test]
        fn get_your_apple_works() {
            let contract = Contract::default();
            let accounts = test::default_accounts::<Environment>();

            // 事前条件：アカウントはりんごを持っていません。

            // アクション：アカウントがりんごを持っているか確認します。
            // アサーション：アカウントはりんごを持っていないので、0が返るはずです。
            assert_eq!(contract.get_your_apple(accounts.alice), 0);
        }

        #[ink::test]
        fn eat_an_apple_works() {
            let mut contract = Contract::default();
            let accounts = test::default_accounts::<Environment>();
            let token_id: Id = Id::U32(1);

             // mint a new token
            assert!(contract.mint(accounts.alice, token_id.clone()).is_ok());

            // 事前条件：まずアカウントに十分なお金を持たせます。
            contract.set_your_money(accounts.alice, 50);

            // 事前条件：まずアカウントにリンゴを持たせます。
            contract.buy_an_apple(accounts.alice).unwrap();

            contract.set_last_eaten(token_id.clone(), 1 * 1000); // 1 second

            set_block_timestamp(6000 * 1000); // 600 seconds (10 minutes)
            
            // アクション：アカウントがリンゴを食べます。
            assert!(contract.eat_an_apple(token_id, accounts.alice).is_ok());
            
            // // アサーション：リンゴを持っていないことを確認します。
            assert_eq!(contract.get_your_apple(accounts.alice), 0);
        }

        #[ink::test]
        fn eat_an_apple_works_without_enough_time() {
            let mut contract = Contract::default();
            let accounts = test::default_accounts::<Environment>();
            let token_id: Id = Id::U32(1);

             // mint a new token
            assert!(contract.mint(accounts.alice, token_id.clone()).is_ok());

            // 事前条件：まずアカウントに十分なお金を持たせます。
            contract.set_your_money(accounts.alice, 50);

            // 事前条件：まずアカウントにリンゴを持たせます。
            contract.buy_an_apple(accounts.alice).unwrap();

            contract.set_last_eaten(token_id.clone(), 590 * 1000); // 590 seconds

            set_block_timestamp(600 * 1000); // 600 seconds (only 10 seconds has passed)
            
            // アクション：アカウントがリンゴを食べます。
            assert!(contract.eat_an_apple(token_id, accounts.alice).is_err());
            
            // // アサーション：リンゴを残っていることを確認します。
            assert_eq!(contract.get_your_apple(accounts.alice), 1);
        }
 
    }
}
