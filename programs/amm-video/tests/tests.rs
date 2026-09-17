use {
    anchor_lang::AccountDeserialize,
    anchor_spl::{associated_token, token::TokenAccount},
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();
    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(ixs, Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), signers).unwrap();
    svm.send_transaction(tx)
}

fn get_token_balance(svm: &LiteSVM, account: &Pubkey) -> u64 {
    let acc = svm.get_account(account).expect("account not found");
    let token_acc = TokenAccount::try_deserialize(&mut acc.data.as_slice()).expect("failed to deserialize token account");
    token_acc.amount
}

// Setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (
    LiteSVM,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_video::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/amm_video.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // Create two mints (Mint X and Mint Y) with 6 decimals and the payer as authority
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let config =
        Pubkey::find_program_address(&[b"config", &123u64.to_le_bytes()], &amm_video::id()).0;
    let mint_lp = Pubkey::find_program_address(&[b"lp", config.as_ref()], &amm_video::id()).0;
    let treasury =
        Pubkey::find_program_address(&[b"treasury", config.as_ref()], &amm_video::id()).0;

    let vault_x = associated_token::get_associated_token_address(&config, &mint_x);
    let vault_y = associated_token::get_associated_token_address(&config, &mint_y);

    let treasury_x = associated_token::get_associated_token_address(&treasury, &mint_x);
    let treasury_y = associated_token::get_associated_token_address(&treasury, &mint_y);

    (
        svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    )
}

#[test]
fn test_initialize() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();

    let instruction = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );
    let res = send(&mut svm, &[instruction], &payer, &[&payer]);
    assert!(res.is_ok());
}

#[test]
fn test_deposit() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    // Check pool vaults have deposited amounts
    assert_eq!(get_token_balance(&svm, &vault_x), 200_000_000);
    assert_eq!(get_token_balance(&svm, &vault_y), 200_000_000);
}

#[test]
fn test_withdraw() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let withdraw_ix = create_withdraw_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );
    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, withdraw_ix],
        &payer,
        &[&payer],
    );
    assert!(res.is_ok());
}

#[test]
fn test_swap_and_fee_to_treasury() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let swap_amount = 10_000_000; // 10 tokens
    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury, treasury_x, treasury_y,
        true, swap_amount, 5_000_000,
    );

    let res = send(&mut svm, &[init_ix, deposit_ix, swap_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    // Fee is 30 bps (0.3%): 10_000_000 * 30 / 10_000 = 30_000
    let treasury_fee_x = get_token_balance(&svm, &treasury_x);
    assert_eq!(treasury_fee_x, 30_000, "Treasury X should have received the swap fee");

    // Vault X should have received 10_000_000 - 30_000 = 9_970_000 net deposit
    let vault_x_balance = get_token_balance(&svm, &vault_x);
    assert_eq!(vault_x_balance, 200_000_000 + (10_000_000 - 30_000));
}

#[test]
fn test_withdraw_fees() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury, treasury_x, treasury_y,
        true, 10_000_000, 5_000_000,
    );

    let withdraw_fees_ix = create_withdraw_fees_ix(
        &mut svm, &payer, mint_x, mint_y, config, treasury, treasury_x, treasury_y, 30_000, 0,
    );

    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, swap_ix, withdraw_fees_ix],
        &payer,
        &[&payer],
    );
    assert!(res.is_ok());

    // Treasury X balance should be 0 after withdrawal
    assert_eq!(get_token_balance(&svm, &treasury_x), 0);
}

#[test]
fn test_lock_and_unlock_pool() {
    let (mut svm, payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y) = setup();
    let init_ix = create_initialise_ix(
        &mut svm, &payer, mint_x, mint_y, config, mint_lp, vault_x, vault_y, treasury, treasury_x, treasury_y,
    );

    let deposit_ix = create_deposit_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y,
    );

    let lock_ix = create_lock_ix(&payer, config);

    // Initialise, deposit, lock pool
    let res = send(&mut svm, &[init_ix, deposit_ix, lock_ix], &payer, &[&payer]);
    assert!(res.is_ok());

    // Swap should FAIL when pool is locked
    let swap_ix = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury, treasury_x, treasury_y,
        true, 10_000_000, 1_000_000,
    );
    let swap_res = send(&mut svm, &[swap_ix], &payer, &[&payer]);
    assert!(swap_res.is_err(), "Swap must fail when pool is locked");

    // Unlock pool
    let unlock_ix = create_unlock_ix(&payer, config);
    let unlock_res = send(&mut svm, &[unlock_ix], &payer, &[&payer]);
    assert!(unlock_res.is_ok());

    // Now swap should succeed!
    let swap_ix2 = create_swap_ix(
        &mut svm, &payer, mint_x, mint_y, mint_lp, config, vault_x, vault_y, treasury, treasury_x, treasury_y,
        true, 10_000_000, 1_000_000,
    );
    let swap_res2 = send(&mut svm, &[swap_ix2], &payer, &[&payer]);
    assert!(swap_res2.is_ok(), "Swap must succeed after unlocking pool");
}
