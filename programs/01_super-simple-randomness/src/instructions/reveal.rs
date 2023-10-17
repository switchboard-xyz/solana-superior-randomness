pub use crate::SbError;
pub use crate::*;

use crate::solana_program::hash::hash;
use solana_program::sysvar::instructions::load_instruction_at_checked;
use solana_program::sysvar::instructions::Instructions;
use anchor_lang::solana_program::sysvar::SysvarId;

#[derive(Accounts)]
pub struct Reveal<'info> {
    #[account(mut)]
    pub req: AccountLoader<'info, RequestAccountData>,
    /// CHECK: ed25519 program
    #[account(address = ed25519_program::ID)]
    pub ed25519_program: AccountInfo<'info>,
    /// CHECK: ix program
    #[account(address = Instructions::id())]
    instruction_sysvar: AccountInfo<'info>,
}

impl Reveal<'_> {
    pub fn reveal(ctx: Context<Reveal>, signature: [u8; 64]) -> anchor_lang::Result<()> {
        let pubkey = ctx.accounts.req.key();
        let mut req = ctx.accounts.req.load_mut()?;

        if req.reveal_timestamp > 0 {
            return Err(error!(SbError::RequestAlreadyRevealed));
        }

        let ix = ed25519_sig_verify_ix(
            &ctx.accounts.ed25519_program,
            &pubkey,
            &signature,
            &req.seed.to_le_bytes(),
        );
        let first_ix = load_instruction_at_checked(0, &ctx.accounts.instruction_sysvar.to_account_info())?;
        if ix != first_ix {
            return Err(error!(SbError::SigVerifyFailed));
        }
        let randomness = hash(&signature).to_bytes();
        req.result = randomness;
        msg!("Randomness-: {:?}", randomness);
        req.reveal_timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }
}
