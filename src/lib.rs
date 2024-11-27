use solana_program::{ 
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use crate::rewards::distribute_reward;
use crate::utils::{get_wallet_token_balance, check_eligibility};

pub mod rewards;
pub mod utils;

#[derive(Debug, Clone)]
pub struct Vote {
    pub voter: Pubkey,
    pub tokens_voted: u64,
    pub reward: u64,
}

entrypoint!(process_instruction);

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let user_account = next_account_info(accounts_iter)?;
    let token_account = next_account_info(accounts_iter)?;
    let dao_account = next_account_info(accounts_iter)?;
    let reward_account = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Validate instruction data length
    if instruction_data.len() < 2 {
        msg!("Invalid instruction data length.");
        return Err(ProgramError::InvalidInstructionData);
    }

    // Match instruction type
    match instruction_data[0] {
        0 => check_eligibility(user_account, token_account, token_program), // Check participation eligibility
        1 => cast_vote(
            user_account,
            token_account,
            reward_account,
            dao_account,
            token_program,
            instruction_data,
            program_id,
        ), // Handle voting
        _ => {
            msg!("Invalid instruction type.");
            Err(ProgramError::InvalidInstructionData)
        }
    }
}

// Cast a vote
pub fn cast_vote(
    user_account: &AccountInfo,
    token_account: &AccountInfo,
    reward_account: &AccountInfo,
    dao_account: &AccountInfo,
    token_program: &AccountInfo,
    instruction_data: &[u8],
    program_id: &Pubkey,
) -> ProgramResult {
    let wallet_address = *user_account.key;
    let token_balance = get_wallet_token_balance(token_account, token_program)?;

    // Check eligibility based on token balance
    if token_balance < 1001 {
        msg!("Wallet {:?} is not eligible to vote.", wallet_address);
        return Err(ProgramError::Custom(2)); // Custom error for voting ineligibility
    }

    // Example logic: Token balance used to calculate voting power
    let vote_percentage = instruction_data[1] as u64; // assuming percentage is passed
    if vote_percentage < 10 || vote_percentage > 25 {
        msg!("Invalid vote percentage.");
        return Err(ProgramError::Custom(3));
    }

    let vote_amount = token_balance * vote_percentage / 100;
    let reward = vote_amount * 30 / 100;

    // Save vote in DAO state (example logic for saving to account data)
    let mut dao_data: Vec<u8> = dao_account.data.borrow().to_vec();
    dao_data.push(vote_amount as u8); // Just an example

    dao_account.data.borrow_mut().copy_from_slice(&dao_data);

    // Distribute rewards
    distribute_reward(wallet_address, reward, reward_account, token_program, program_id)?;

    Ok(())
}
