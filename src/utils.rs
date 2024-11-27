use solana_program::{account_info::AccountInfo, program_error::ProgramError};
use spl_token::state::Account as TokenAccount;

// Helper function to get wallet token balance
pub fn get_wallet_token_balance(
    token_account: &AccountInfo,
    token_program: &AccountInfo,
) -> Result<u64, ProgramError> {
    let token_data = TokenAccount::unpack(&token_account.data.borrow())?;
    if *token_account.owner != *token_program.key {
        msg!("Provided account is not a valid token account.");
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(token_data.amount)
}

// Check DAO membership eligibility
pub fn check_eligibility(
    user_account: &AccountInfo,
    token_account: &AccountInfo,
    token_program: &AccountInfo,
) -> ProgramResult {
    let token_balance = get_wallet_token_balance(token_account, token_program)?;

    if token_balance < 1001 || token_balance > 250000 {
        msg!("Wallet {:?} is not eligible to participate.", user_account.key);
        return Err(ProgramError::Custom(1)); // Custom error for invalid eligibility
    }

    msg!("Wallet {:?} is eligible to participate.", user_account.key);
    Ok(())
}
