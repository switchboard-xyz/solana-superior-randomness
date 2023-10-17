pub mod instructions;
pub use instructions::*;

pub use anchor_lang::prelude::*;
pub use switchboard_solana::*;

pub use bytemuck;
pub use bytemuck::{Pod, Zeroable};

declare_id!("BbaDMQdYvxkpH6g9oKJ5wx2H2UK5P31mpaKTngXtjMhE");

#[account(zero_copy(unsafe))]
#[repr(packed)]
pub struct RequestAccountData {
    pub bump: u8,
    pub pubkey_hash: [u8; 32],
    pub switchboard_request: Pubkey,
    pub seed: u32,
    pub blockhash: [u8; 32],
    pub result: [u8; 32],
    pub request_timestamp: i64,
    pub seed_timestamp: i64,
    pub reveal_timestamp: i64,
}

#[program]
pub mod superior_randomness {
    use super::*;

    pub fn request(ctx: Context<Request>, keyhash: [u8; 32]) -> anchor_lang::Result<()> {
        Request::request(ctx, keyhash)
    }

    pub fn seed(ctx: Context<Seed>, seed: u32) -> anchor_lang::Result<()> {
        Seed::seed(ctx, seed)
    }

    pub fn reveal(ctx: Context<Reveal>, pubkey: Pubkey) -> anchor_lang::Result<()> {
        Reveal::reveal(ctx, pubkey)
    }
}

#[error_code]
pub enum SbError {
    RequestAlreadySeeded,
    RequestAlreadyRevealed,
    KeyVerifyFailed,
}
