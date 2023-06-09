#![cfg_attr(not(feature = "std"), no_std)]
#![feature(min_specialization)]

#[openbrush::contract]
pub mod my_psp34_mintable {
    use openbrush::{
        contracts::psp34::extensions::mintable::*,
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
        #[storage_field]
        psp34: psp34::Data,

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

        // #[ink(message)]
        // pub fn ensure_exists_and_get_owner(&self, id: Id) -> Result<AccountId, String> {
        //     let token_owner = self
        //         .data::<psp34::Data<enumerable::Balances>>()
        //         .owner_of(id.clone())
        //         .ok_or(PSP34Error::TokenNotExists)?;
        //     Ok(token_owner)
        // }

        #[ink(message)]
        pub fn set_status (
            &mut self,
            token_id: Id, 
            hungry: u32,
            health: u32,
            happy: u32
        ) -> Result<(), String>{ 
            // こちら、実装する必要あり！！！
            // self.ensure_exists_and_get_owner(&token_id)?;
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
        pub fn get_status(&self, token_id: Id) -> Option<Status> {
            self.asset_status.get(&token_id)
        }     

        
    }
}
