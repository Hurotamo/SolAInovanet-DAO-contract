use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use spl_token::state::Account as TokenAccount;
use crate::ProgramError;

pub fn get_wallet_token_balance(
    token_account: &AccountInfo,
    token_program: &AccountInfo,
) -> Result<u64, ProgramError> {
    let token_data = TokenAccount::unpack(&token_account.data.borrow())?;
    if *token_account.owner != *token_program.key {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(token_data.amount)
}

pub fn validate_voter_balance(
    token_account: &AccountInfo,
    token_program: &AccountInfo,
    min_balance: u64,
    excluded_balance: u64,
) -> Result<u64, ProgramError> {
    let balance = get_wallet_token_balance(token_account, token_program)?;
    if balance < min_balance || balance <= excluded_balance {
        return Err(ProgramError::Custom(2)); // Custom error for ineligible voter
    }
    Ok(balance)
}
