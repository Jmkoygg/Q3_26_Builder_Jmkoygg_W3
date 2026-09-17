use {
    anchor_lang::{
        solana_program::instruction::Instruction, system_program::ID as SYSTEM_PROGRAM_ID,
        InstructionData, ToAccountMetas,
    },
    anchor_spl::associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
    litesvm::LiteSVM,
    litesvm_token::spl_token::ID as TOKEN_PROGRAM_ID,
    solana_keypair::Keypair,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
};

pub fn create_withdraw_fees_ix(
    mut _svm: &mut LiteSVM,
    authority: &Keypair,
    mint_x: Pubkey,
    mint_y: Pubkey,
    config: Pubkey,
    treasury: Pubkey,
    treasury_x: Pubkey,
    treasury_y: Pubkey,
    amount_x: u64,
    amount_y: u64,
) -> Instruction {
    let auth = authority.pubkey();
    let authority_x = associated_token::get_associated_token_address(&auth, &mint_x);
    let authority_y = associated_token::get_associated_token_address(&auth, &mint_y);

    Instruction::new_with_bytes(
        amm_video::id(),
        &amm_video::instruction::WithdrawFees {
            amount_x,
            amount_y,
        }
        .data(),
        amm_video::accounts::WithdrawFees {
            authority: auth,
            mint_x,
            mint_y,
            config,
            treasury,
            treasury_x,
            treasury_y,
            authority_x,
            authority_y,
            token_program: TOKEN_PROGRAM_ID,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
    )
}
