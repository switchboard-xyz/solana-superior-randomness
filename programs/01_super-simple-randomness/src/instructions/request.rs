pub use crate::SbError;
pub use crate::*;

#[derive(Accounts)]
pub struct Request<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        space = 8 + std::mem::size_of::<RequestAccountData>(),
        payer = payer,
    )]
    pub req: AccountLoader<'info, RequestAccountData>,

    /// CHECK:
    pub authority: AccountInfo<'info>,

    /// CHECK: Switchboard attestation program
    #[account(executable, address = SWITCHBOARD_ATTESTATION_PROGRAM_ID)]
    pub switchboard: AccountInfo<'info>,

    pub switchboard_state: AccountLoader<'info, AttestationProgramState>,
    pub switchboard_attestation_queue: AccountLoader<'info, AttestationQueueAccountData>,
    #[account(mut)]
    pub switchboard_function: AccountLoader<'info, FunctionAccountData>,
    /// CHECK: validated by Switchboard CPI
    #[account(
        mut,
        signer,
        owner = system_program.key(),
        constraint = switchboard_request.lamports() == 0
      )]
    pub switchboard_request: AccountInfo<'info>,
    /// CHECK:
    #[account(
        mut,
        owner = system_program.key(),
        constraint = switchboard_request_escrow.lamports() == 0
      )]
    pub switchboard_request_escrow: AccountInfo<'info>,

    // TOKEN ACCOUNTS
    #[account(address = anchor_spl::token::spl_token::native_mint::ID)]
    pub switchboard_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    // SYSTEM ACCOUNTS
    pub system_program: Program<'info, System>,
}
impl Request<'_> {
    pub fn request(ctx: Context<Request>) -> anchor_lang::Result<()> {
        let mut req = ctx.accounts.req.load_init()?;

        // https://docs.rs/switchboard-solana/latest/switchboard_solana/attestation_program/instructions/request_init_and_trigger/index.html
        let request_init_ctx = FunctionRequestInitAndTrigger {
            request: ctx.accounts.switchboard_request.clone(),
            authority: ctx.accounts.authority.to_account_info(),
            function: ctx.accounts.switchboard_function.to_account_info(),
            function_authority: None,
            escrow: ctx.accounts.switchboard_request_escrow.clone(),
            mint: ctx.accounts.switchboard_mint.to_account_info(),
            state: ctx.accounts.switchboard_state.to_account_info(),
            attestation_queue: ctx.accounts.switchboard_attestation_queue.to_account_info(),
            payer: ctx.accounts.payer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
            token_program: ctx.accounts.token_program.to_account_info(),
            associated_token_program: ctx.accounts.associated_token_program.to_account_info(),
        };
        let params = format!("PID={},REQUEST_KEY={}", crate::ID, ctx.accounts.req.key());

        request_init_ctx.invoke_signed(
            ctx.accounts.switchboard.clone(),
            // bounty - optional fee to reward oracles for priority processing
            // default: 0 lamports
            None,
            // slots_until_expiration - optional max number of slots the request can be processed in
            // default: 2250 slots, ~ 15 min at 400 ms/slot
            // minimum: 150 slots, ~ 1 min at 400 ms/slot
            None,
            // max_container_params_len - the length of the vec containing the container params
            // default: 256 bytes
            Some(512),
            // container_params - the container params
            // default: empty vec
            Some(params.into()),
            // garbage_collection_slot - the slot when the request can be closed by anyone and is considered dead
            // default: None, only authority can close the request
            None,
            // valid_after_slot - schedule a request to execute in N slots
            // default: 0 slots, valid immediately for oracles to process
            None,
            // signer seeds
            &[],
        )?;

        req.request_timestamp = Clock::get()?.unix_timestamp;
        req.switchboard_request = ctx.accounts.switchboard_request.key();

        Ok(())
    }
}
