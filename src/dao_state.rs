use solana_program::{pubkey::Pubkey};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DAOState {
    pub qualified_wallets: Vec<Pubkey>,
    pub votes: Vec<Vote>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Vote {
    pub voter: Pubkey,
    pub tokens_voted: u64,
    pub reward: u64,
}

impl DAOState {
    pub fn add_vote(&mut self, voter: Pubkey, tokens_voted: u64, reward: u64) {
        self.votes.push(Vote { voter, tokens_voted, reward });
    }

    pub fn is_qualified(&self, wallet: &Pubkey) -> bool {
        self.qualified_wallets.contains(wallet)
    }
}
