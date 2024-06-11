//! Solana program runtime program tools.

use {
    solana_bpf_loader_program::syscalls::create_program_runtime_environment_v1,
    solana_program_runtime::{
        compute_budget::ComputeBudget,
        invoke_context::BuiltinFunctionWithContext,
        loaded_programs::{LoadProgramMetrics, LoadedProgram, LoadedProgramsForTxBatch},
    },
    solana_sdk::{
        account::{Account, AccountSharedData},
        bpf_loader_upgradeable::{self, UpgradeableLoaderState},
        feature_set::FeatureSet,
        native_loader,
        pubkey::Pubkey,
        rent::Rent,
    },
    std::{
        fs::File,
        io::Read,
        path::{Path, PathBuf},
        sync::Arc,
    },
};

struct Builtin {
    program_id: Pubkey,
    name: &'static str,
    entrypoint: BuiltinFunctionWithContext,
}

static AGAVE_BUILTINS: &[Builtin] = &[
    Builtin {
        program_id: solana_system_program::id(),
        name: "system_program",
        entrypoint: solana_system_program::system_processor::Entrypoint::vm,
    },
    Builtin {
        program_id: solana_sdk::bpf_loader_upgradeable::id(),
        name: "solana_bpf_loader_upgradeable_program",
        entrypoint: solana_bpf_loader_program::Entrypoint::vm,
    },
    /* ... */
];

pub fn system_program_account(rent: &Rent) -> AccountSharedData {
    let data = AGAVE_BUILTINS[0].name.as_bytes().to_vec();
    let lamports = rent.minimum_balance(data.len());
    AccountSharedData::from(Account {
        lamports,
        data,
        owner: native_loader::id(),
        executable: true,
        rent_epoch: 0,
    })
}

pub fn program_account(program_id: &Pubkey, rent: &Rent) -> AccountSharedData {
    let programdata_address =
        Pubkey::find_program_address(&[program_id.as_ref()], &bpf_loader_upgradeable::id()).0;
    let data = bincode::serialize(&UpgradeableLoaderState::Program {
        programdata_address,
    })
    .unwrap();
    AccountSharedData::from(Account {
        lamports: rent.minimum_balance(data.len()).max(1),
        data,
        owner: bpf_loader_upgradeable::id(),
        executable: true,
        rent_epoch: 0,
    })
}

pub fn build_program_cache() -> LoadedProgramsForTxBatch {
    let mut cache = LoadedProgramsForTxBatch::default();

    AGAVE_BUILTINS.iter().for_each(
        |Builtin {
             program_id,
             name,
             entrypoint,
         }| {
            cache.replenish(
                *program_id,
                Arc::new(LoadedProgram::new_builtin(0, name.len(), *entrypoint)),
            );
        },
    );

    cache
}

pub fn add_program_to_cache(
    cache: &mut LoadedProgramsForTxBatch,
    program_id: &Pubkey,
    program_name: &'static str,
    compute_budget: &ComputeBudget,
    feature_set: &FeatureSet,
) {
    let elf = load_program_elf(program_name);
    let program_runtime_environment =
        create_program_runtime_environment_v1(feature_set, compute_budget, false, false).unwrap();

    cache.replenish(
        *program_id,
        Arc::new(
            LoadedProgram::new(
                &bpf_loader_upgradeable::id(),
                Arc::new(program_runtime_environment),
                0,
                0,
                None,
                &elf,
                elf.len(),
                &mut LoadProgramMetrics::default(),
            )
            .unwrap(),
        ),
    );
}

fn load_program_elf(program_name: &str) -> Vec<u8> {
    let program_file = find_file(&format!("{program_name}.so"))
        .expect("Program file data not available for {program_name}");
    read_file(program_file)
}

fn read_file<P: AsRef<Path>>(path: P) -> Vec<u8> {
    let path = path.as_ref();
    let mut file = File::open(path)
        .unwrap_or_else(|err| panic!("Failed to open \"{}\": {}", path.display(), err));

    let mut file_data = Vec::new();
    file.read_to_end(&mut file_data)
        .unwrap_or_else(|err| panic!("Failed to read \"{}\": {}", path.display(), err));
    file_data
}

fn find_file(filename: &str) -> Option<PathBuf> {
    for dir in default_shared_object_dirs() {
        let candidate = dir.join(filename);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

fn default_shared_object_dirs() -> Vec<PathBuf> {
    let mut search_path = vec![];
    if let Ok(bpf_out_dir) = std::env::var("BPF_OUT_DIR") {
        search_path.push(PathBuf::from(bpf_out_dir));
    } else if let Ok(bpf_out_dir) = std::env::var("SBF_OUT_DIR") {
        search_path.push(PathBuf::from(bpf_out_dir));
    }
    search_path.push(PathBuf::from("tests/fixtures"));
    if let Ok(dir) = std::env::current_dir() {
        search_path.push(dir);
    }
    search_path
}
