use anchor_lang::prelude::*;

declare_id!("6U7ezSr7phBBojC5PRUuutUNFpMiDxUxXeiKfjTZduMs");

#[program]
pub mod solana_tipjar {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
