use anchor_lang::prelude::*;

use crate::{error::AmmError, state::Config};

#[derive(Accounts)]
pub struct LockPool<'info> {
    pub authority: Signer<'info>,
    #[account(
        mut,
        seeds = [b"config", config.seed.to_le_bytes().as_ref()],
        bump = config.config_bump,
    )]
    pub config: Account<'info, Config>,
}

impl<'info> LockPool<'info> {
    pub fn lock(&mut self) -> Result<()> {
        require!(
            self.config.authority == Some(self.authority.key()),
            AmmError::Unauthorized
        );
        self.config.locked = true;
        Ok(())
    }

    pub fn unlock(&mut self) -> Result<()> {
        require!(
            self.config.authority == Some(self.authority.key()),
            AmmError::Unauthorized
        );
        self.config.locked = false;
        Ok(())
    }
}
