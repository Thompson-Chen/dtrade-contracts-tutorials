#![cfg_attr(not(feature = "std"), no_std)]

/// solidity version of this contract 
/// could be found here: https://solidity.readthedocs.io/en/v0.5.3/solidity-by-example.html
/// there are some modifications made to this contract

use ink_lang as ink;

#[ink::contract]
mod ballot {
    use ink_storage::collections::HashMap;
    /// make sure to include ink_prelude as dependency in cargo.toml file
    use ink_prelude::string::String;
    use ink_prelude::vec::Vec;
    use ink_storage::traits::{PackedLayout, SpreadLayout};


    #[derive(Clone, Debug, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    struct Proposal {
        name: String, // Name of proposal
        vote_count: u32, // number of accumulated votes
    }

    #[derive(Copy, Clone, Debug, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Voter {
        weight: u32, // weight is accumulated by delegation
        voted: bool,  // if true, that person already voted
        delegate: Option<AccountId>, // person delegated to
        vote: Option<i32>,   // index of the voted proposal
    }

    /// ballot to store the chair person name, voters and proposals
    #[cfg(not(feature = "ink-as-dependency"))]
    #[ink(storage)]
    pub struct Ballot {
        chair_person: AccountId,
        voters: HashMap<AccountId, Voter>,
        proposals: Vec<Proposal>,
    }

    impl Ballot {

        /// If proposal names are provided, they are saved to Ballot
        /// else tha bllot is empty
        /// The person who deploys the contract is the chairperson
        #[ink(constructor)]
        pub fn new(proposal_names: Option<Vec<String>> )-> Self {

            // get chair person address
            let chair_person =  Self::env().caller();

            // create empty propsal and voters
            let mut proposals: Vec<Proposal> = Vec::new();
            let mut voters = HashMap::new();

            // initialize chain person's vote
            voters.insert(chair_person, Voter{
                weight:1,
                voted:false,
                delegate: None,
                vote: None,
            });

            // if proposals are provided
            if proposal_names.is_some() {
                // store the provided propsal names
                let names = proposal_names.unwrap();
                for name in &names {
                    proposals.push(
                        Proposal{
                        name: String::from(name),
                        vote_count: 0,
                    });
                }
            }

            Self {
                chair_person,
                voters,
                proposals,
            }
        }

        /// default constrcutor
        #[ink(constructor)]
        pub fn default() -> Self {
            Self::new(Default::default())
        }
        
        /// Simply returns the current chairperson id`.
        #[ink(message)]
        pub fn get_chairperson(&self) -> AccountId {
            self.chair_person
        }

        /// Give `voter` the right to vote on this ballot.
        // May only be called by `chairperson`.
        #[ink(message)]
        pub fn give_voting_right(&mut self, voter_id: AccountId) {
            let caller = self.env().caller();

            // only chair person can give right to vote
            assert_eq!(caller,self.chair_person, "only chair person can give right to vote");

            let voter_opt = self.voters.get_mut(&voter_id);
            // the voter does not exists
            assert_eq!(voter_opt.is_some(),true, "provided voterId does not exist");

            let voter = voter_opt.unwrap();
            // the voter should not have already voted
            assert_eq!(voter.voted,false, "the voter has already voted");

            voter.weight = 1;
        }

        /// the function adds the provided voter id into possible
        /// list of voters. By default the voter has no voting right,
        /// the contract owner must approve the voter before he can cast a vote
        #[ink(message)]
        pub fn add_voter(&mut self, voter_id: AccountId) -> bool{

            let voter_opt = self.voters.get(&voter_id);
            // the voter does not exists
            if voter_opt.is_some() {
                return false
            }

            self.voters.insert(voter_id, Voter{
                weight:0,
                voted:false,
                delegate: None,
                vote: None,
            });
            return true
        }

        /// Delegate your vote to the voter `to`.
        /// If the `to` has already voted, you vote is casted to
        /// the same candidate as `to`
        #[ink(message)]
        pub fn delegate(&mut self, to: AccountId)  {

            // account id of the person who invoked the function
            let sender_id = self.env().caller();
            let sender_weight;
            assert_ne!(to,sender_id, "Self-delegation is disallowed.");

            {
                let sender_opt =  self.voters.get_mut(&sender_id);
                assert_eq!(sender_opt.is_some(),true, "Caller is not a valid voter");
                let sender = sender_opt.unwrap();

                assert_eq!(sender.voted,false, "You have already voted");

                sender.voted = true;
                sender.delegate = Some(to);
                sender_weight = sender.weight;
            }

            {
                let delegate_opt = self.voters.get_mut(&to);
                assert_eq!(delegate_opt.is_some(),true, "The delegated address is not valid");

                let delegate = delegate_opt.unwrap();

                // the voter should not have already voted
                if delegate.voted {
                    // If the delegate already voted,
                    // directly add to the number of votes
                    let voted_to = delegate.vote.unwrap() as usize;
                    self.proposals[voted_to].vote_count += sender_weight;
                } else {
                    // If the delegate did not vote yet,
                    // add to her weight.
                    delegate.weight += sender_weight;
                }
            }
        }

        /// Give your vote (including votes delegated to you)
        /// to proposal `proposals[proposal].name`.
        #[ink(message)]
        pub fn vote(&mut self, proposal_index: i32) {
            let sender_id = self.env().caller();

            let sender_opt =  self.voters.get_mut(&sender_id);
            assert_eq!(sender_opt.is_some(),true, "Sender is not a voter!");

            // the voter should not have already voted
            let sender = sender_opt.unwrap();
            assert_eq!(sender.voted,false, "You have already voted");

            assert_eq!(sender.weight,1, "You have no right to vote");

            // get the proposal
            let proposal_opt = self.proposals.get_mut(proposal_index as usize);
            assert_eq!(proposal_opt.is_some(),true, "Proposal index out of bound");

            let proposal = proposal_opt.unwrap();

            sender.voted = true;
            sender.vote = Some(proposal_index);

            proposal.vote_count += sender.weight;
        }

        /// @dev Computes the winning proposal taking all
        /// previous votes into account.
        fn winning_proposal(&self) -> Option<usize> {
            let mut winning_vote_vount:u32 = 0;
            let mut winning_index: Option<usize> = None;
            let mut index: usize = 0;

            for val in self.proposals.iter() {
                if val.vote_count > winning_vote_vount {
                    winning_vote_vount = val.vote_count;
                    winning_index = Some(index);
                }
                index += 1

            }
            return winning_index
        }

        // Calls winning_proposal() function to get the index
        // of the winner contained in the proposals array and then
        // returns the name of the winner
        pub fn get_winning_proposal_name(&self) -> &String {
            let winner_index: Option<usize> = self.winning_proposal();
            assert_eq!(winner_index.is_some(),true, "No Proposal!");
            let index = winner_index.unwrap();
            let proposal = self.proposals.get(index).unwrap();
            return &proposal.name

        }

        /// given an index returns the name of the proposal at that index
        pub fn get_proposal_name_at_index(&self, index:usize) -> &String {
            let proposal = self.proposals.get(index).unwrap();
            return &proposal.name
        }

        /// returns the number of proposals in ballet
        pub fn get_proposal_length(&self) -> usize {
            return self.proposals.len()
        }

        /// adds the given proposal name in ballet
        /// to do: check unqiueness of proposal,
        pub fn add_proposal(&mut self, proposal_name: String){
            self.proposals.push(
                Proposal{
                    name:String::from(proposal_name),
                    vote_count: 0,
            });
        }

        pub fn get_voter(&self, voter_id: AccountId) -> Option<&Voter>{
            self.voters.get(&voter_id)
        }



    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_lang as ink;

        #[ink::test]
        fn new_works() {
            let mut proposal_names: Vec<String> = Vec::new();
            proposal_names.push(String::from("Proposal # 1"));  
            let ballot = Ballot::new(Some(proposal_names));
            assert_eq!(ballot.get_proposal_length(),1);
        }

        #[ink::test]
        fn default_works() {
            let ballot = Ballot::default();
            assert_eq!(ballot.get_proposal_length(), 0);
        }

        #[ink::test]
        fn adding_proposals_works() {
            let mut ballot = Ballot::default();
            ballot.add_proposal(String::from("Proposal #1"));
            assert_eq!(ballot.get_proposal_length(),1);
        }

        #[ink::test]
        fn adding_voters_work() {
            let mut ballot = Ballot::default();
            let account_id = AccountId::from([0x0; 32]);
            assert_eq!(ballot.add_voter(account_id),true);
            assert_eq!(ballot.add_voter(account_id),false);
        }

        #[ink::test]
        fn give_voting_rights_work() {
            let mut ballot = Ballot::default();
            let account_id = AccountId::from([0x0; 32]);

            ballot.add_voter(account_id);     
            ballot.give_voting_right(account_id);
            let voter = ballot.get_voter(account_id).unwrap();
            assert_eq!(voter.weight,1);
        }

        #[ink::test]
        fn voting_works() {
            let mut ballot = Ballot::default();
            ballot.add_proposal(String::from("Proposal #1"));
            ballot.vote(0);
            let voter = ballot.get_voter(ballot.get_chairperson()).unwrap();
            assert_eq!(voter.voted,true);
        }

        #[ink::test]
        fn delegation_works() {
            let mut ballot = Ballot::default();
            let to_id = AccountId::from([0x0; 32]);

            ballot.add_voter(to_id);     
            ballot.delegate(to_id);

            let voter = ballot.get_voter(ballot.get_chairperson()).unwrap();
            assert_eq!(voter.delegate.unwrap(),to_id);
        } 

        #[ink::test]
        fn get_winning_proposal_name_working() {
            let mut ballot = Ballot::default();
            ballot.add_proposal(String::from("Proposal #1"));
            ballot.add_proposal(String::from("Proposal #2"));
            ballot.vote(0);
            let proposal_name = ballot.get_winning_proposal_name();
            assert_eq!(proposal_name, "Proposal #1");
        }
        
    }
}
