use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid token account")]
    InvalidToken,
    #[msg("Authority mismatched")]
    AuthorityMismatch,
    #[msg("Account has illegal owner")]
    IllegalOwner,
}