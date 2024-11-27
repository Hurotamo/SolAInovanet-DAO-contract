use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};
use spl_token::state::Account as TokenAccount;

// Constants
const MIN_TOKENS: u64 = 1001;
const MAX_TOKENS: u64 = 250000;
const VOTING_MIN_BALANCE: u64 = 1001;
const VOTING_EXCLUDED_BALANCE: u64 = 2500;
const REWARD_PERCENTAGE: u64 = 30;

// DAO State
#[derive(Debug, Clone)]
pub struct DAOState {
    pub qualified_wallets: Vec<Pubkey>,
    pub votes: Vec<Vote>,
}

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
        0 => check_membership(user_account, token_account, token_program), // Check participation eligibility
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

// Check DAO membership eligibility
pub fn check_membership(
    user_account: &AccountInfo,
    token_account: &AccountInfo,
    token_program: &AccountInfo,
) -> ProgramResult {
    let wallet_address = *user_account.key;
    let token_balance = get_wallet_token_balance(token_account, token_program)?;

    if token_balance < MIN_TOKENS || token_balance > MAX_TOKENS {
        msg!(
            "Wallet address {:?} is not eligible to participate. Token balance: {}",
            wallet_address,
            token_balance
        );
        return Err(ProgramError::Custom(1)); // Custom error code for invalid eligibility
    }

    msg!("Wallet address {:?} is eligible to participate.", wallet_address);
    Ok(())
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

    if token_balance < VOTING_MIN_BALANCE || token_balance <= VOTING_EXCLUDED_BALANCE {
        msg!("Wallet address {:?} is not eligible to vote.", wallet_address);
        return Err(ProgramError::Custom(2)); // Custom error for voting ineligibility
    }

    let vote_percentage = instruction_data[1] as u64; // Assuming instruction_data[1] holds percentage
    if vote_percentage < 10 || vote_percentage > 25 {
        msg!(
            "Invalid vote percentage for wallet {:?}. Provided: {}%",
            wallet_address,
            vote_percentage
        );
        return Err(ProgramError::Custom(3)); // Custom error for invalid percentage
    }

    let vote_amount = token_balance * vote_percentage / 100;
    let reward = vote_amount * REWARD_PERCENTAGE / 100;

    // Save vote in DAO state (on-chain data storage)
    let mut dao_state: DAOState = bincode::deserialize(&dao_account.data.borrow())?;
    dao_state.votes.push(Vote {
        voter: wallet_address,
        tokens_voted: vote_amount,
        reward,
    });
    dao_account.data.borrow_mut().copy_from_slice(&bincode::serialize(&dao_state)?);

    // Distribute rewards
    distribute_reward(wallet_address, reward, reward_account, token_program, program_id)?;

    msg!(
        "Wallet {:?} successfully voted with {} tokens. Reward: {} tokens.",
        wallet_address,
        vote_amount,
        reward
    );

    Ok(())
}

// Helper function to get wallet token balance
fn get_wallet_token_balance(
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

// Secure reward distribution
fn distribute_reward(
    wallet_address: Pubkey,
    reward: u64,
    reward_account: &AccountInfo,
    token_program: &AccountInfo,
    program_id: &Pubkey,
) -> ProgramResult {
    if reward == 0 {
        msg!("No reward to distribute.");
        return Ok(());
    }

    let seeds = &[b"reward-seed", &[reward_account.lamports() as u8]];
    let signer = &[&seeds[..]];
    invoke_signed(
        &spl_token::instruction::transfer(
            token_program.key,
            reward_account.key,
            &wallet_address,
            reward_account.key,
            &[],
            reward,
        )?,
        &[reward_account.clone()],
        signer,
    )?;

    msg!(
        "Distributed {} tokens as reward to wallet {:?}.",
        reward,
        wallet_address
    );

    Ok(())
}
