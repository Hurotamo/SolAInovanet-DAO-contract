use solana_program::{account_info::{next_account_info, AccountInfo}, entrypoint::ProgramResult, pubkey::Pubkey, msg};
use crate::{account_management::validate_voter_balance, dao_state::{DAOState, Vote}, rewards::distribute_reward};

pub fn check_membership(
    user_account: &AccountInfo,
    token_account: &AccountInfo,
    token_program: &AccountInfo,
) -> ProgramResult {
    // Check membership eligibility logic
    let balance = validate_voter_balance(token_account, token_program, MIN_TOKENS, MAX_TOKENS)?;
    msg!("Account {:?} has {} tokens and is eligible for membership.", user_account.key, balance);
    Ok(())
}

pub fn cast_vote(
    user_account: &AccountInfo,
    token_account: &AccountInfo,
    reward_account: &AccountInfo,
    dao_account: &AccountInfo,
    token_program: &AccountInfo,
    vote_percentage: u64,
) -> ProgramResult {
    // Validate balance and vote percentage logic
    let balance = validate_voter_balance(token_account, token_program, VOTING_MIN_BALANCE, VOTING_EXCLUDED_BALANCE)?;

    let vote_amount = balance * vote_percentage / 100;
    let reward = vote_amount * REWARD_PERCENTAGE / 100;

    // Update DAO state
    let mut dao_state: DAOState = bincode::deserialize(&dao_account.data.borrow())?;
    dao_state.add_vote(*user_account.key, vote_amount, reward);

    // Save updated DAO state
    dao_account.data.borrow_mut().copy_from_slice(&bincode::serialize(&dao_state)?);

    // Distribute reward
    distribute_reward(*user_account.key, reward, reward_account, token_program)?;

    msg!("Vote cast successfully with {} tokens. Reward: {} tokens", vote_amount, reward);
    Ok(())
}
