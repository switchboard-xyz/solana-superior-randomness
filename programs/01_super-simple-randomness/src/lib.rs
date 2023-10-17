pub mod instructions;
pub use instructions::*;

pub mod ed25519;
pub use ed25519::*;

pub use anchor_lang::prelude::*;
pub use switchboard_solana::*;

pub use bytemuck;
pub use bytemuck::{Pod, Zeroable};

pub use solana_program::ed25519_program;

declare_id!("CoFRoANHRszvCi5VEUw2DxoTU5UAq5oXskjK3SSxz2i");

#[account(zero_copy(unsafe))]
#[repr(packed)]
pub struct RequestAccountData {
    pub switchboard_request: Pubkey,
    pub seed: u32,
    pub result: [u8; 32],
    pub request_timestamp: i64,
    pub seed_timestamp: i64,
    pub reveal_timestamp: i64,
}

#[program]
pub mod superior_randomness {
    use super::*;

    pub fn request(ctx: Context<Request>) -> anchor_lang::Result<()> {
        Request::request(ctx)
    }

    pub fn seed(ctx: Context<Seed>, seed: u32) -> anchor_lang::Result<()> {
        Seed::seed(ctx, seed)
    }

    pub fn reveal(ctx: Context<Reveal>, signature: [u8; 64]) -> anchor_lang::Result<()> {
        Reveal::reveal(ctx, signature)
    }
}

#[error_code]
pub enum SbError {
    RequestAlreadySeeded,
    RequestAlreadyRevealed,
    SigVerifyFailed,
}
