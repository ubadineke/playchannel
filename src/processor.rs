//! A helper to initialize Solana SVM API's `TransactionBatchProcessor`.

use {
    solana_bpf_loader_program::syscalls::create_program_runtime_environment_v1,
    solana_compute_budget::compute_budget::ComputeBudget,
    solana_program_runtime::loaded_programs::{
        BlockRelation, ForkGraph, LoadProgramMetrics, ProgramCacheEntry,
    },
    solana_sdk::{
        account::ReadableAccount, clock::Slot, feature_set::FeatureSet, pubkey::Pubkey, transaction,
    },
    solana_svm::{
        account_loader::CheckedTransactionDetails,
        transaction_processing_callback::TransactionProcessingCallback,
        transaction_processor::TransactionBatchProcessor,
    },
    solana_system_program::system_processor,
    std::{
        fs,
        sync::{Arc, RwLock},
    },
};

/// In order to use the `TransactionBatchProcessor`, another trait - Solana
/// Program Runtime's `ForkGraph` - must be implemented, to tell the batch
/// processor how to work across forks.
///
/// Since PayTube doesn't use slots or forks, this implementation is mocked.
pub(crate) struct PayTubeForkGraph {}

impl ForkGraph for PayTubeForkGraph {
    fn relationship(&self, _a: Slot, _b: Slot) -> BlockRelation {
        BlockRelation::Unknown
    }
}

/// This function encapsulates some initial setup required to tweak the
/// `TransactionBatchProcessor` for use within PayTube.
///
/// We're simply configuring the mocked fork graph on the SVM API's program
/// cache, then adding the System program to the processor's builtins.
pub(crate) fn create_transaction_batch_processor<CB: TransactionProcessingCallback>(
    callbacks: &CB,
    feature_set: &FeatureSet,
    compute_budget: &ComputeBudget,
) -> TransactionBatchProcessor<PayTubeForkGraph> {
    let processor = TransactionBatchProcessor::<PayTubeForkGraph>::default();

    {
        let mut cache = processor.program_cache.write().unwrap();
        let sysvar_cache =  processor.sysvar_cache();
        // sysvar_cache.get_clock().unwrap();
        sysvar_cache.get_rent().expect("Failed at getting rent sysvar");
        // sysvar_cache.get_last_restart_slot().unwrap();
        // sysvar_cache.get_epoch_rewards().unwrap();
        // sysvar_cache.get_epoch_schedule().unwrap();
        // sysvar_cache.get_slot_hashes().unwrap();
        // sysvar_cache.

        // clock: Option<Vec<u8>>,
        // epoch_schedule: Option<Vec<u8>>,
        // epoch_rewards: Option<Vec<u8>>,
        // rent: Option<Vec<u8>>,
        // slot_hashes: Option<Vec<u8>>,
        // stake_history: Option<Vec<u8>>,
        // last_restart_slot:

        // Initialize the mocked fork graph.
        cache.fork_graph = Some(Arc::new(RwLock::new(PayTubeForkGraph {})));

        // Initialize a proper cache environment.
        // (Use Loader v4 program to initialize runtime v2 if desired)
        cache.environments.program_runtime_v1 = Arc::new(
            create_program_runtime_environment_v1(feature_set, compute_budget, false, false)
                .unwrap(),
        );

        // Add the SPL Token program to the cache.
        if let Some(program_account) = callbacks.get_account_shared_data(&spl_token::id()) {
            let elf_bytes = program_account.data();
            let program_runtime_environment = cache.environments.program_runtime_v1.clone();
            cache.assign_program(
                spl_token::id(),
                Arc::new(
                    ProgramCacheEntry::new(
                        &solana_sdk::bpf_loader::id(),
                        program_runtime_environment,
                        0,
                        0,
                        elf_bytes,
                        elf_bytes.len(),
                        &mut LoadProgramMetrics::default(),
                    )
                    .unwrap(),
                ),
            );
        }

        //Add Rock Paper Scissors to Cache
        let program_id: Pubkey = "B6iwgaDVFX7LXDMokCYT8Ya21gr2FbsUTBPFh2mcfxNa"
            .parse()
            .unwrap();
        let data = fs::read(
            std::env::current_dir()
                .unwrap()
                .join("rock_paper_scissors.so")
                .to_str()
                .unwrap(),
        )
        .expect("read program binary");
        let elf_bytes: &[u8] = &data;
        let program_runtime_environment = cache.environments.program_runtime_v1.clone();
        cache.assign_program(
            program_id,
            Arc::new(
                ProgramCacheEntry::new(
                    &solana_sdk::bpf_loader::id(),
                    program_runtime_environment,
                    0,
                    0,
                    elf_bytes,
                    elf_bytes.len(),
                    &mut LoadProgramMetrics::default(),
                )
                .unwrap(),
            ),
        );
    }

    // Add the system program builtin.
    processor.add_builtin(
        callbacks,
        solana_system_program::id(),
        "system_program",
        ProgramCacheEntry::new_builtin(
            0,
            b"system_program".len(),
            system_processor::Entrypoint::vm,
        ),
    );

    // Add the BPF Loader v2 builtin, for the SPL Token program.
    processor.add_builtin(
        callbacks,
        solana_sdk::bpf_loader::id(),
        "solana_bpf_loader_program",
        ProgramCacheEntry::new_builtin(
            0,
            b"solana_bpf_loader_program".len(),
            solana_bpf_loader_program::Entrypoint::vm,
        ),
    );

    processor
}

/// This functions is also a mock. In the Agave validator, the bank pre-checks
/// transactions before providing them to the SVM API. We mock this step in
/// PayTube, since we don't need to perform such pre-checks.
pub(crate) fn get_transaction_check_results(
    len: usize,
    lamports_per_signature: u64,
) -> Vec<transaction::Result<CheckedTransactionDetails>> {
    vec![
        transaction::Result::Ok(CheckedTransactionDetails {
            nonce: None,
            lamports_per_signature,
        });
        len
    ]
}
