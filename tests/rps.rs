mod setup;

use std::fs;

use borsh::BorshSerialize;
// use mollusk_svm::Mollusk;
use paytube_svm::{
    game_traits::GameMove,
    games::{register_builtin_games, rock_paper_scissors::RPSChoice},
    transaction_two::{Choice, RpsTransaction},
    PlayChannel,
};
use setup::{system_account, TestValidatorContext};
use solana_sdk::{
    account::{Account, AccountSharedData},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
};
use std::fs::File;
use std::io::Read;
use std::path::Path;

#[test]
fn test_rps() {
    let uba = Keypair::new();
    let clem = Keypair::new();

    let uba_pubkey = uba.pubkey();
    let clem_pubkey = clem.pubkey();

    let program_id: Pubkey = "B6iwgaDVFX7LXDMokCYT8Ya21gr2FbsUTBPFh2mcfxNa"
        .parse()
        .unwrap();

    let accounts = vec![
        (uba_pubkey, system_account(10_000_000)),
        (clem_pubkey, system_account(10_000_000)),
        (program_account(
            program_id,
            std::env::current_dir()
                .unwrap()
                .join("rock_paper_scissors.so")
                .to_str()
                .unwrap(),
        )),
    ];

    let context = TestValidatorContext::start_with_accounts(accounts);
    let test_validator = &context.test_validator;
    let payer = context.payer.insecure_clone();

    let rpc_client = test_validator.get_rpc_client();
    // let program_account = rpc_client.get_account(&program_id).unwrap();
    // println!("Program owner: {}", program_account.owner);
    // println!(
    //     "Expected owner: {}",
    //     solana_sdk::bpf_loader_upgradeable::id()
    // );

    // println!("Program account: {:?}", program_account);

    // match rpc_client.get_account(&program_id) {
    //     Ok(account) => {
    //         if account.executable {
    //             println!("Program {} exists and is executable!", program_id);
    //         } else {
    //             println!("Account {} exists but is NOT executable", program_id);
    //         }
    //     }
    //     Err(err) => {
    //         println!("Program not found: {}", err);
    //     }
    // }

    //Create a channel
    let play_channel = PlayChannel::new(
        vec![payer, uba.insecure_clone(), clem.insecure_clone()],
        rpc_client,
    );
    let game_pda = Pubkey::find_program_address(
        &[b"game", &uba_pubkey.to_bytes(), &2u64.to_le_bytes()],
        &program_id,
    );
    println!("{}", game_pda.0);

    play_channel.process_plays(&[
        RpsTransaction {
            game: game_pda.0,
            player: uba_pubkey,
            player_two: Some(clem_pubkey),
            choice: Choice::Paper,
            program_id,
        },
        //     RpsTransaction {
        //     game: game_pda.0,
        //     player: uba_pubkey,
        //     choice: Choice::Paper,
        //     program_id,
        // },
        // RpsTransaction {
        //     game: game_pda.0,
        //     player: uba_pubkey,
        //     choice: Choice::Paper,
        //     program_id,
        // }
    ]);

    assert!(false)
    // register_builtin_games(play_channel.game_manager.get_mut_registry());

    // let game_instance_id = play_channel
    //     .create_game(
    //         &"rock_paper_scissors".to_string(),
    //         &[uba_pubkey, clem_pubkey],
    //         0,
    //         None,
    //     )
    //     .unwrap();

    // let game_choice = RPSChoice::Paper.try_to_vec().unwrap();

    // let uba_move = GameMove {
    //     game_instance_id,
    //     player_id: uba_pubkey,
    //     move_id: 1,
    //     move_data: game_choice.clone(),
    //     signature: uba.sign_message(&game_choice),
    //     timestamp: std::time::SystemTime::now()
    //         .duration_since(std::time::UNIX_EPOCH)
    //         .unwrap()
    //         .as_secs(),
    // };

    // // let result = play_channel.process_game_move(&uba_move).unwrap();

    // dbg!(game_instance_id);
    // dbg!(result);No
}

pub fn program_account(program_id: Pubkey, program_path: &str) -> (Pubkey, AccountSharedData) {
    // Load compiled .so
    let elf_bytes = read_file(Path::new(program_path));

    // Create an executable account owned by the loader
    let account = Account {
        lamports: 1_000_000_000,
        data: elf_bytes,
        owner: solana_sdk::bpf_loader_upgradeable::id(), // or bpf_loader_upgradeable::id()
        executable: true,
        rent_epoch: 0,
    };

    (program_id, AccountSharedData::from(account))
}

pub fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let path = path.as_ref();
    let mut file = File::open(path).unwrap();

    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data).unwrap();
    file_data
}
