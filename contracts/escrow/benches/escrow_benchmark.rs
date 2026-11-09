//! Gas / CPU benchmarks for the `escrow` contract.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p escrow --bench escrow_benchmark
//! ```
//!
//! This follows the custom-harness pattern the other contract benchmarks use
//! (#871, #873, #989): a `harness = false` binary registered in `Cargo.toml`,
//! so `cargo bench` really runs on the pinned stable toolchain.
//!
//! The previous version of this file used nightly `#[bench]` / `test::Bencher`,
//! imported a crate that does not exist (`escrow_contract`) and passed a random
//! `Address` as the config contract that `create_escrow` invokes, so it was
//! never compilable — let alone runnable — on the pinned toolchain.
//!
//! Reported per call, after resetting the host budget meters:
//!
//! - `wall_ns` — mean wall-clock time on this host;
//! - `cpu` — host CPU instructions per call;
//! - `mem` — host memory bytes per call.
//!
//! `cpu` and `mem` are host figures, not on-chain instruction counts; `soroban-sdk`
//! 21 exposes no public cost-estimate API. Wall-clock numbers are
//! machine-dependent and informational only.

extern crate std;

use std::time::Instant;

use soroban_sdk::{
    contract, contractimpl, symbol_short,
    testutils::Address as _,
    token, Address, Bytes, Env,
};

use escrow::{EscrowContract, EscrowContractClient};

const ITERATIONS: u32 = 25;
const AMOUNT: i128 = 1_000;
const FEE_BPS: u32 = 250;

/// Minimal stand-in for `platform_config`.
///
/// `escrow` reaches the platform config through `invoke_contract` and only
/// needs three readers, so the benchmark registers this instead of a real
/// config contract.
#[contract]
pub struct MockConfig;

#[contractimpl]
impl MockConfig {
    pub fn init(env: Env, fee_bps: u32, usdc: Address, admin: Address, platform_wallet: Address) {
        env.storage().instance().set(&symbol_short!("fee"), &fee_bps);
        env.storage().instance().set(&symbol_short!("usdc"), &usdc);
        env.storage().instance().set(&symbol_short!("adm"), &admin);
        env.storage().instance().set(&symbol_short!("pw"), &platform_wallet);
    }

    pub fn get_fee_b(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("fee"))
            .unwrap_or(0)
    }

    pub fn get_usdc(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("usdc"))
            .unwrap()
    }

    pub fn get_adm(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("adm"))
            .unwrap()
    }

    pub fn get_pw(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&symbol_short!("pw"))
            .unwrap()
    }
}

struct Fixture {
    env: Env,
    client: EscrowContractClient<'static>,
    config: Address,
    payer: Address,
    artist: Address,
}

fn fixture() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();
    env.budget().reset_unlimited();

    let admin = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();

    let config_id = env.register_contract(None, MockConfig);
    let platform_wallet = Address::generate(&env);
    MockConfigClient::new(&env, &config_id).init(&FEE_BPS, &token, &admin, &platform_wallet);

    let escrow_id = env.register_contract(None, EscrowContract);
    let client = EscrowContractClient::new(&env, &escrow_id);

    let payer = Address::generate(&env);
    let artist = Address::generate(&env);
    // Every create/release/refund cycle moves `AMOUNT`, and each scenario may
    // need its own funded escrow.
    token::StellarAssetClient::new(&env, &token).mint(
        &payer,
        &(AMOUNT * ITERATIONS as i128 * 4),
    );

    Fixture {
        env,
        client,
        config: config_id,
        payer,
        artist,
    }
}

fn escrow_id(env: &Env, tag: &str, i: u32) -> Bytes {
    Bytes::from_slice(env, std::format!("{tag}-{i}").as_bytes())
}

/// Pre-create one escrow per iteration so the measured call is the operation
/// under test rather than setup.
fn funded_escrows(f: &Fixture, tag: &str) -> std::vec::Vec<Bytes> {
    let mut ids = std::vec::Vec::with_capacity(ITERATIONS as usize);
    for i in 0..ITERATIONS {
        let id = escrow_id(&f.env, tag, i);
        f.client
            .create_escrow(&id, &f.payer, &f.artist, &AMOUNT, &f.config);
        ids.push(id);
    }
    ids
}

struct Row {
    op: &'static str,
    wall_ns: u128,
    cpu: u64,
    mem: u64,
}

fn run<F: FnMut(u32)>(env: &Env, op: &'static str, mut f: F) -> Row {
    let mut budget = env.budget();
    budget.reset_unlimited();
    let start = Instant::now();
    for i in 0..ITERATIONS {
        f(i);
    }
    let elapsed = start.elapsed().as_nanos();
    Row {
        op,
        wall_ns: elapsed / ITERATIONS as u128,
        cpu: budget.cpu_instruction_cost() / ITERATIONS as u64,
        mem: budget.memory_bytes_cost() / ITERATIONS as u64,
    }
}

fn bench_create_escrow() -> Row {
    let f = fixture();
    run(&f.env, "create_escrow", |i| {
        let id = escrow_id(&f.env, "create", i);
        f.client
            .create_escrow(&id, &f.payer, &f.artist, &AMOUNT, &f.config);
    })
}

fn bench_open_dispute() -> Row {
    let f = fixture();
    let ids = funded_escrows(&f, "dispute");
    run(&f.env, "open_dispute", |i| {
        f.client.open_dispute(&ids[i as usize], &f.payer);
    })
}

fn bench_release_payment() -> Row {
    let f = fixture();
    let ids = funded_escrows(&f, "release");
    run(&f.env, "release_payment", |i| {
        f.client.release_payment(&ids[i as usize], &f.config);
    })
}

fn bench_refund_client() -> Row {
    let f = fixture();
    let ids = funded_escrows(&f, "refund");
    run(&f.env, "refund_client", |i| {
        f.client.refund_client(&ids[i as usize], &f.config);
    })
}

fn bench_get_escrow() -> Row {
    let f = fixture();
    let id = escrow_id(&f.env, "get", 0);
    f.client
        .create_escrow(&id, &f.payer, &f.artist, &AMOUNT, &f.config);
    run(&f.env, "get_escrow", |_| {
        let _ = f.client.get_escrow(&id);
    })
}

fn main() {
    let rows = [
        bench_create_escrow(),
        bench_open_dispute(),
        bench_release_payment(),
        bench_refund_client(),
        bench_get_escrow(),
    ];

    println!();
    println!("escrow — warm-call budget (n = {ITERATIONS})");
    println!("{:<20} {:>12} {:>14} {:>12}", "op", "wall ns", "cpu", "mem");
    for row in &rows {
        println!(
            "{:<20} {:>12} {:>14} {:>12}",
            row.op, row.wall_ns, row.cpu, row.mem
        );
    }
    println!();
    println!("wall ns is machine-dependent; cpu/mem are host figures, not on-chain counts");
}
