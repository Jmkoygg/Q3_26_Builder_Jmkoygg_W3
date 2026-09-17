use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Default Error")]
    DefaultError,
    #[msg("Offer expired")]
    OfferExpired,
    #[msg("Price is zero")]
    PriceIsZero,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Pool is locked")]
    PoolLocked,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid fee amount")]
    InvalidFee,
}
