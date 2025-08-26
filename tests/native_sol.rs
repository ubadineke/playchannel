mod setup;

use {
    paytube_svm::{transaction::PayTubeTransaction, PayTubeChannel},
    setup::{system_account, TestValidatorContext},
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    },
    std::fs,
};

#[test]
fn test_native_sol() {
    let alice = Keypair::new();
    let bob = Keypair::new();
    let will = Keypair::new();

    let alice_pubkey = alice.pubkey();
    let bob_pubkey = bob.pubkey();
    let will_pubkey = will.pubkey();
    let program_id: Pubkey = "B6iwgaDVFX7LXDMokCYT8Ya21gr2FbsUTBPFh2mcfxNa"
        .parse()
        .unwrap();

    let accounts = vec![
        (alice_pubkey, system_account(10_000_000)),
        (bob_pubkey, system_account(10_000_000)),
        (will_pubkey, system_account(10_000_000)),
        // (program_account(
        //     program_id,
        //     std::env::current_dir()
        //         .unwrap()
        //         .join("rock_paper_scissors.so")
        //         .to_str()
        //         .unwrap(),
        // )),
    ];

    let context = TestValidatorContext::start_with_accounts(accounts);
    let test_validator = &context.test_validator;
    let payer = context.payer.insecure_clone();

    let rpc_client = test_validator.get_rpc_client();

    match rpc_client.get_account(&program_id) {
        Ok(account) => {
            if account.executable {
                println!("Program {} exists and is executable!", program_id);
            } else {
                println!("Account {} exists but is NOT executable", program_id);
            }
        }
        Err(err) => {
            println!("Program not found: {}", err);
        }
    }

    let paytube_channel = PayTubeChannel::new(vec![payer, alice, bob, will], rpc_client);

    paytube_channel.process_paytube_transfers(&[
        // Alice -> Bob 2_000_000
        PayTubeTransaction {
            from: alice_pubkey,
            to: bob_pubkey,
            amount: 2_000_000,
            mint: None,
        },
        // Bob -> Will 5_000_000
        PayTubeTransaction {
            from: bob_pubkey,
            to: will_pubkey,
            amount: 5_000_000,
            mint: None,
        },
        // Alice -> Bob 2_000_000
        PayTubeTransaction {
            from: alice_pubkey,
            to: bob_pubkey,
            amount: 2_000_000,
            mint: None,
        },
        // Will -> Alice 1_000_000
        PayTubeTransaction {
            from: will_pubkey,
            to: alice_pubkey,
            amount: 1_000_000,
            mint: None,
        },
    ]);

    // Ledger:
    // Alice:   10_000_000 - 2_000_000 - 2_000_000 + 1_000_000  = 7_000_000
    // Bob:     10_000_000 + 2_000_000 - 5_000_000 + 2_000_000  = 9_000_000
    // Will:    10_000_000 + 5_000_000 - 1_000_000              = 14_000_000
    let rpc_client = test_validator.get_rpc_client();
    assert_eq!(rpc_client.get_balance(&alice_pubkey).unwrap(), 7_000_000);
    assert_eq!(rpc_client.get_balance(&bob_pubkey).unwrap(), 9_000_000);
    assert_eq!(rpc_client.get_balance(&will_pubkey).unwrap(), 14_000_000);
}

pub fn program_account(program_id: Pubkey, program_path: &str) -> (Pubkey, AccountSharedData) {
    // Load compiled .so
    let elf_bytes = fs::read(program_path).expect("read program binary");

    // Create an executable account owned by the loader
    let account = Account {
        lamports: 1_000_000_000, // must have rent exemption
        data: elf_bytes,
        owner: solana_sdk::bpf_loader::id(), // or bpf_loader_upgradeable::id()
        executable: true,
        rent_epoch: 0,
    };

    (program_id, AccountSharedData::from(account))
}
