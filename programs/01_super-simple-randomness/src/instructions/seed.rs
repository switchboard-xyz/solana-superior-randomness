pub use crate::SbError;
pub use crate::*;
use arrayref::array_ref;

#[event]
pub struct RequestSeededEvent {
    pub request: Pubkey,
    pub seed: u32,
}

#[derive(Accounts)]
pub struct Seed<'info> {
    #[account(
        mut,
        has_one = switchboard_request,
    )]
    pub req: AccountLoader<'info, RequestAccountData>,

    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    #[account(
    constraint = switchboard_request.validate_signer(
        &switchboard_function.to_account_info(),
        &enclave_signer.to_account_info()
        )?
    )]
    pub switchboard_request: Box<Account<'info, FunctionRequestAccountData>>,
    pub enclave_signer: Signer<'info>,
    /// CHECK: todo
    #[account(address = solana_program::sysvar::recent_blockhashes::ID)]
    pub recent_blockhashes: AccountInfo<'info>,
}
impl Seed<'_> {
    pub fn seed(ctx: Context<Seed>, seed: u32) -> anchor_lang::Result<()> {
        let mut req = ctx.accounts.req.load_mut()?;

        if req.seed_timestamp > 0 {
            return Err(error!(SbError::RequestAlreadySeeded));
        }
        let block_data = ctx.accounts.recent_blockhashes.data.borrow();

        req.seed = seed;
        req.seed_timestamp = Clock::get()?.unix_timestamp;
        req.blockhash = array_ref![block_data, 8, 32].clone();

        emit!(RequestSeededEvent {
            request: ctx.accounts.req.key(),
            seed,
        });
        Ok(())
    }
}
