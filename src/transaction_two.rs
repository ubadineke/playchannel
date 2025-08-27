//! PayTube's custom transaction format, tailored specifically for SOL or SPL
//! token transfers.
//!
//! Mostly for demonstration purposes, to show how projects may use completely
//! different transactions in their protocol, then convert the resulting state
//! transitions into the necessary transactions for the base chain - in this
//! case Solana.

use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_sdk::{
        instruction::{AccountMeta, Instruction as SolanaInstruction},
        pubkey::Pubkey,
        system_instruction, system_program,
        transaction::{
            SanitizedTransaction as SolanaSanitizedTransaction, Transaction as SolanaTransaction,
        },
    },
    spl_associated_token_account::get_associated_token_address,
    std::collections::HashSet,
};

/// A custom Rock-Paper-Scissors transaction.
/// This is not a SOL/SPL transfer, but an *action* inside the game.
pub struct RpsTransaction {
    pub game: Pubkey,   // PDA for the game state account
    pub player: Pubkey, // The signer making the move
    pub player_two: Option<Pubkey>,
    pub choice: Choice, // 0 = Rock, 1 = Paper, 2 = Scissors,
    pub program_id: Pubkey,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Choice {
    Rock,
    Paper,
    Scissors,
}

impl From<&RpsTransaction> for SolanaInstruction {
    fn from(value: &RpsTransaction) -> Self {
        let RpsTransaction {
            game,
            player,
            player_two,
            choice,
            program_id,
        } = value;

        // let discriminator: [u8; 8] = [207, 18, 251, 32, 135, 122, 160, 77];
        // let mut ix_data = discriminator.to_vec();
        // let choice_data = borsh::to_vec(choice).unwrap();
        // ix_data.extend_from_slice(&choice_data);

        // SolanaInstruction {
        //     program_id: *program_id,
        //     accounts: vec![
        //         AccountMeta::new(*game, false),
        //         AccountMeta::new(*player, true),
        //     ],
        //     data: ix_data,
        // }

        let discriminator2 = [44, 62, 102, 247, 126, 208, 130, 215];
        let mut ix_data = discriminator2.to_vec();
        let player_two_data = borsh::to_vec(&player_two.unwrap()).unwrap();
        let game_id = borsh::to_vec(&2u64).unwrap();
        // let game_id = &2u64.to_le_bytes()
        ix_data.extend_from_slice(&player_two_data);
        ix_data.extend_from_slice(&game_id);

        // let raw_data = borsh::

        print!("Game PDA: {}", game);
        SolanaInstruction {
            program_id: *program_id,
            accounts: vec![
                AccountMeta::new(*game, false),
                AccountMeta::new(*player, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: ix_data,
        }

        // system_instruction::transfer(player, &player_two.unwrap(), 200000)
        // if let Some(mint) = mint {
        //     let source_pubkey = get_associated_token_address(from, mint);
        //     let destination_pubkey = get_associated_token_address(to, mint);
        //     return spl_token::instruction::transfer(
        //         &spl_token::id(),
        //         &source_pubkey,
        //         &destination_pubkey,
        //         from,
        //         &[],
        //         *amount,
        //     )
        //     .unwrap();
        // }
        // system_instruction::transfer(from, to, *amount)
    }
}

impl From<&RpsTransaction> for SolanaTransaction {
    fn from(value: &RpsTransaction) -> Self {
        SolanaTransaction::new_with_payer(&[SolanaInstruction::from(value)], Some(&value.player))
    }
}

impl From<&RpsTransaction> for SolanaSanitizedTransaction {
    fn from(value: &RpsTransaction) -> Self {
        SolanaSanitizedTransaction::try_from_legacy_transaction(
            SolanaTransaction::from(value),
            &HashSet::new(),
        )
        .unwrap()
    }
}

/// Create a batch of Solana transactions, for the Solana SVM's transaction
/// processor, from a batch of PayTube instructions.
pub fn create_svm_transactions(
    paytube_transactions: &[RpsTransaction],
) -> Vec<SolanaSanitizedTransaction> {
    paytube_transactions
        .iter()
        .map(SolanaSanitizedTransaction::from)
        .collect()
}
