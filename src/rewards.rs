use solana_program::{
    account_info::{AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke_signed,
};
use spl_token::instruction::transfer;
use solana_program::pubkey::Pubkey;

// Secure reward distribution logic
pub fn distribute_reward(
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

    // Construct seed for signature
    let seeds = &[b"reward-seed", &[reward_account.lamports() as u8]];
    let signer = &[&seeds[..]];

    invoke_signed(
        &transfer(
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

    msg!("Distributed {} tokens as reward to wallet {:?}.", reward, wallet_address);

    Ok(())
}
