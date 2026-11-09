#![no_std]
pub mod admin;
pub mod config;
pub mod errors;
pub mod guard;
pub mod pause;
pub mod types;
pub mod upgrade;
pub mod health;
pub mod rollout;
pub mod correlation;
pub mod circuit_breaker;
pub mod pagination;
pub mod dispute;
pub mod initialization;

pub use health::{
    AlertConfig, HealthMetrics, HealthReport, HealthStatus, SlaTargets,
};
pub use pagination::{
    clamp_limit, collect_window, page_info, paginate, paginated, window, PageInfo, DEFAULT_LIMIT,
    MAX_LIMIT,
};
pub use rollout::{
    flag_cap, set_flag_cap, RolloutError, RolloutPhase, RolloutState, MAX_FEATURE_FLAGS,
};
pub use admin::{
    AdminError, AdminKey, AdminRotatedEvent, AdminTransferCancelledEvent,
    AdminTransferProposedEvent,
};
pub use circuit_breaker::{
    CircuitBreakerConfig, CircuitBreakerError, CircuitBreakerState,
};
pub mod validation;
pub mod version;

/// Minimal contract used by the unit tests in this crate.
///
/// Instance storage is only reachable from inside a deployed contract, and the
/// host requires the instance ledger entry to already exist, so [`tests`] run
/// their bodies against this contract instead of a bare generated address.
#[cfg(test)]
#[soroban_sdk::contract]
pub struct TestContract;

#[cfg(test)]
#[soroban_sdk::contractimpl]
impl TestContract {}
