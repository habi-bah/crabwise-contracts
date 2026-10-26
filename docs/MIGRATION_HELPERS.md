# Migration Helpers

This document describes the recommended patterns for handling storage
migrations in crabwise contracts.

## Background

Soroban contracts manage persistent storage with `Env::storage()`. Once a
contract is deployed, its storage schema cannot be updated in place.
Instead, the contract must be upgraded (via `env.deployer().update_current_contract_wasm`)
and the new code must handle both old and new storage layouts.

## Strategies

### 1. Lazy Migration (Recommended)

Write the new data key alongside the old one during write operations. On read,
check for the new key first; if absent, fall back to the old key and optionally
migrate on the fly.

```rust
pub fn get_config(env: &Env) -> ConfigV2 {
    if env.storage().instance().has(&DataKey::ConfigV2) {
        env.storage().instance().get(&DataKey::ConfigV2).unwrap()
    } else {
        // Migrate from V1 on first access
        let old: ConfigV1 = env.storage().instance().get(&DataKey::ConfigV1).unwrap();
        let new = ConfigV2 {
            admin: old.admin,
            fee_bps: old.fee_bps,
            platform_wallet: old.platform_wallet,
            usdc_token: old.usdc_token,
            extra_param: Default::default(),
        };
        env.storage().instance().set(&DataKey::ConfigV2, &new);
        new
    }
}
```

### 2. Batch Migration (Off-Chain)

For contracts with many stored entries (e.g., per-escrow records), write an
admin-only migration function that iterates over all known keys in a
pagination loop.

```rust
pub fn migrate_storage(env: Env, start_cursor: u32, batch_size: u32) -> u32 {
    let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
    admin.require_auth();
    // iterate over a known range and upgrade each record
    // return the number of migrated items
}
```

Due to Soroban ledger limits, batch migrations must be called multiple times
with different cursors through an off-chain script.

### 3. Version-Checked Read

Store a `MIGRATION_VERSION` constant. Every read function checks the version
and applies cumulative migrations if behind:

```rust
fn ensure_migrated(env: &Env) {
    let version: u32 = env.storage()
        .instance().get(&DataKey::MigrationVersion).unwrap_or(0);
    if version < 1 { migrate_v0_to_v1(env); }
    if version < 2 { migrate_v1_to_v2(env); }
    // always write the latest version
    env.storage().instance().set(&DataKey::MigrationVersion, &2u32);
}
```

## Testing Migrations

1. **Unit tests**: Deploy the old contract, write storage in the old format,
   upgrade to the new contract, and assert reads return the expected data.
2. **Integration tests**: Use `Env::register_contract` and `Env::deployer()`
   to simulate a full upgrade lifecycle.

## Pre-Deployment Checklist

- [ ] Increment `MIGRATION_VERSION` in the new contract
- [ ] Write a read-side fallback for each old key
- [ ] Test the upgrade path with a mock old-storage snapshot
- [ ] Verify that old keys can be garbage-collected (or kept for backward compatibility)
