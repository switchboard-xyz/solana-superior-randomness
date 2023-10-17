pub use crate::SbError;
pub use crate::*;

use crate::solana_program::hash::hash;

#[derive(Accounts)]
pub struct Reveal<'info> {
    #[account(mut)]
    pub req: AccountLoader<'info, RequestAccountData>,
}

impl Reveal<'_> {
    pub fn reveal(ctx: Context<Reveal>, pubkey: Pubkey) -> anchor_lang::Result<()> {
        let mut req = ctx.accounts.req.load_mut()?;

        if req.reveal_timestamp > 0 {
            return Err(error!(SbError::RequestAlreadyRevealed));
        }

        let seed = req.seed.to_le_bytes();
        let blockhash = req.blockhash;

        if hash(&pubkey.to_bytes()).to_bytes() != req.pubkey_hash {
            return Err(error!(SbError::KeyVerifyFailed));
        }
        let randomness = hash(&[pubkey.to_bytes().to_vec(), blockhash.to_vec(), seed.to_vec()].concat()).to_bytes();
        req.result = randomness;
        msg!("Randomness-: {:?}", randomness);
        req.reveal_timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }
}
