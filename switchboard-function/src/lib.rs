use std::result::Result;
use switchboard_solana::{prelude::*, solana_client::rpc_client::RpcClient};

// The program ID doesnt matter here because the method were using
// to fetch an account doesnt check the account owner and only checks
// the discriminator which only depends on the AccountName.

declare_id!("11111111111111111111111111111111");


pub async fn load_account<T: bytemuck::Pod + Discriminator>(
    client: &solana_client::rpc_client::RpcClient,
    pubkey: Pubkey,
    program_id: Pubkey,
) -> Result<T, SbError> {
    let account = client
        .get_account(&pubkey)
        .map_err(|_| SbError::CustomMessage("AnchorParseError".to_string()))?;

    if account.owner != program_id {
        return Err(SbError::CustomMessage(
            "Account is not owned by this program".to_string(),
        ));
    }

    if account.data.len() < T::discriminator().len() {
        return Err(SbError::CustomMessage(
            "no discriminator found".to_string(),
        ));
    }

    let mut disc_bytes = [0u8; 8];
    disc_bytes.copy_from_slice(&account.data[..8]);
    if disc_bytes != T::discriminator() {
        return Err(SbError::CustomMessage(
            "Discriminator error, check the account type".to_string(),
        ));
    }

    Ok(*bytemuck::try_from_bytes::<T>(&account.data[8..])
        .map_err(|_| SbError::CustomMessage("AnchorParseError".to_string()))?)
}

