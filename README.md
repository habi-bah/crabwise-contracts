# crabwise-contracts

**Soroban smart contracts for Crabwise, a Rust learning community on Stellar where tutors teach, learners earn XLM for code that actually passes, and study groups put their own stake behind finishing.**

This repo holds the on-chain half of Crabwise. It handles tutor staking, course sales with revenue splits, live-session escrow, test-verified learner rewards (Proof of Compile), and stake-backed study cohorts (Molt Pods).

![License: MIT](https://img.shields.io/badge/License-MIT-green) ![Network: Stellar](https://img.shields.io/badge/Network-Stellar-blue) ![Contracts: Soroban](https://img.shields.io/badge/Contracts-Soroban-purple) ![Language: Rust](https://img.shields.io/badge/Language-Rust-orange)

> **Status:** early design and Testnet development. The function tables, invariants, and error codes below describe the target interface. Items marked *planned* are not yet implemented.

**Part of Crabwise:** [crabwise-web](https://github.com/your-org/crabwise-web) (Next.js app) · [crabwise-api](https://github.com/your-org/crabwise-api) (API, runner, AI service) · crabwise-contracts (this repo)

---

## What is Crabwise?

Learning Rust is hard, and most learners quit at the borrow checker. Crabwise connects tutors and learners in one place:

- Tutors publish video courses and run live sessions. Learners buy, download, and learn offline.
- Learners write Rust in the browser and **earn XLM when their code passes the tutor's tests**.
- A social feed and forum keep the community talking, and learners can trade on Stellar from inside the app.
- An AI coach explains compiler errors, gives hints before answers, and summarises the latest crypto news with sources.

### The two unique features

1. **Proof of Compile.** Challenges pay learners automatically when a sandboxed runner attests that their solution passes the tutor's tests. No manual grading, no waiting.
2. **Molt Pods.** Small cohorts stake a few XLM against weekly goals. Finishers get their stake back plus a share of what dropouts forfeit, and part of the forfeits funds scholarships. (A crab molts to grow, hence the name.)

---

## Contracts

| Contract | Purpose |
|---|---|
| `TutorRegistry` | Tutor profiles, stake, verification, suspension, and verified-purchase ratings |
| `CourseMarket` | Course listings, immutable content versions, purchases held for a dispute window, revenue splits, access checks, and scholarship grants |
| `SessionEscrow` | Escrow for live 1:1 sessions with cancellation rules, delivery confirmation, and disputes |
| `CompileVault` | Proof of Compile: funded challenges, runner attestations, and learner rewards |
| `MoltPod` | Molt Pods: staked study cohorts, progress attestations, settlement, and scholarship forfeits |

```
crabwise-contracts/
├── contracts/
│   ├── tutor-registry/
│   ├── course-market/
│   ├── session-escrow/
│   ├── compile-vault/
│   ├── molt-pod/
│   └── common/              # Shared error enum and types
├── tests/                   # Cross-contract integration tests
├── scripts/                 # Testnet deploy and bootstrap
├── docs/                    # SECURITY_MODEL.md, ARCHITECTURE.md, EVENTS.md
├── Cargo.toml
└── README.md                <- you are here
```
**Stack:** Rust, Soroban SDK, Stellar CLI

**What stays off-chain:** videos, source code, chat, posts, and the sandbox that runs tests. The ledger holds only hashes, prices, stakes, attestations, and money flows.

---

## TutorRegistry

| Function | Purpose |
|---|---|
| `initialize(admin, verifiers, min_stake)` | Sets the admin, approved verifiers, and the minimum tutor stake |
| `register(tutor, profile_hash, stake)` | Registers a tutor and locks their stake. Tutor auth required |
| `verify(tutor, attestation_hash)` | An approved verifier confirms a tutor (for example, after a passing advanced challenge or reviewed credentials) |
| `suspend(tutor, reason_hash)` / `reinstate(tutor)` | Verifier or admin suspends or reinstates a tutor |
| `add_stake(tutor, amount)` / `withdraw_stake(tutor, amount)` | Manages stake. Withdrawal is blocked while the tutor has open sessions, disputes, or a suspension |
| `slash(tutor, amount, reason_hash)` | Slashes stake for confirmed misconduct. Admin multisig |
| `record_rating(tutor, learner, item_id, rating_bps)` | Called only by `CourseMarket` or `SessionEscrow` to record a rating after a real purchase or session. One rating per (learner, item) |
| `lock_for_session(tutor)` / `unlock_for_session(tutor)` | Called only by `SessionEscrow` to track open sessions |
| `get_tutor(tutor) -> TutorInfo` | Read-only |
| `is_active(tutor) -> bool` | Read-only. True if verified, not suspended, and above minimum stake |

## CourseMarket

| Function | Purpose |
|---|---|
| `__constructor(MarketConfig)` | Sets admin, token, split in basis points (tutor, platform, scholarship), dispute window, arbiters, and contract addresses in one call |
| `create_course(tutor, metadata_hash, price) -> u64` | Lists a course. The tutor must be active. Returns `course_id` |
| `publish_version(course_id, content_hash)` | Adds an immutable content version. Earlier versions stay visible |
| `set_price(course_id, price)` | Changes the price for future purchases only |
| `delist(course_id)` | Stops new purchases. Existing access is unaffected |
| `purchase(buyer, course_id) -> u64` | Pays the price into escrow and returns `purchase_id`. Access is granted immediately |
| `release(purchase_id)` | After the dispute window, pays out the split to the tutor, platform, and scholarship balance. Publicly callable |
| `open_dispute(purchase_id, evidence_hash)` | Buyer disputes within the window. Freezes the payout |
| `resolve_dispute(purchase_id, refund_bps)` | An arbiter resolves with a full, partial, or no refund |
| `rate(purchase_id, rating_bps)` | Buyer rates after purchase. Forwards to `TutorRegistry` |
| `grant_access(learner, course_id, reason_hash)` | The scholarship committee grants access, paid from the scholarship balance |
| `deposit_scholarship(from, amount)` | Adds funds to the scholarship balance. Called by `MoltPod` and anyone who donates |
| `has_access(learner, course_id) -> bool` | Read-only. True for paid or granted access, and not refunded |
| `get_course(course_id) -> Course` / `get_purchase(purchase_id) -> Purchase` | Read-only |
| `get_user_nonce(caller) -> u64` | Read-only replay-protection nonce |
| `pause()` / `unpause()` | Emergency stop. Admin auth required |

## SessionEscrow

| Function | Purpose |
|---|---|
| `__constructor(SessionConfig)` | Sets admin, token, arbiters, timeouts, late-cancel penalty, and the `TutorRegistry` address |
| `book(learner, tutor, amount, start_ledger, duration_ledgers) -> u64` | Locks funds for a session. Returns `session_id`. The tutor must be active |
| `accept(session_id)` | The tutor accepts the booking |
| `cancel(session_id)` | Learner or tutor cancels. Learner gets a full refund before the cancel deadline and a partial refund after it. A tutor cancellation always refunds in full |
| `mark_delivered(session_id)` | The tutor marks the session as delivered |
| `confirm(session_id)` | The learner confirms and releases payment. Optional rating |
| `auto_release(session_id)` | If the learner stays silent past the confirm timeout, funds release to the tutor. Publicly callable |
| `claim_no_show(session_id)` | If the tutor never delivers, the learner refunds in full and the tutor's stake can be slashed |
| `open_dispute(session_id, evidence_hash)` / `resolve_dispute(session_id, learner_bps)` | Dispute and arbiter resolution |
| `get_session(session_id) -> Session` | Read-only |

## CompileVault (Proof of Compile)

| Function | Purpose |
|---|---|
| `__constructor(CompileConfig)` | Sets admin, token, runner set, runner threshold, minimum budget, and the `CourseMarket` address in one call |
| `create_challenge(author, tests_hash, starter_hash, reward, budget, max_winners, end_ledger, course_id) -> u64` | Funds a challenge. `course_id` is optional; when set, only learners with course access can claim |
| `submit(learner, challenge_id, solution_hash, nonce)` | Records a submission. The learner's code is sent to the runner off-chain |
| `attest(challenge_id, learner, solution_hash, result_hash, passed, score_bps)` | A registered runner attests the result of running the tests. Runner auth required |
| `claim(challenge_id, learner)` | Pays the reward if a passing attestation exists. One reward per learner per challenge |
| `close_challenge(challenge_id)` | After the end ledger, returns unspent budget to the author |
| `add_runner(runner)` / `remove_runner(runner)` / `rotate_runner(old, new)` | Admin multisig manages runner keys |
| `report_plagiarism(challenge_id, learner, evidence_hash)` | Arbiter voids a reward and sanctions the learner |
| `get_challenge(challenge_id) -> Challenge` | Read-only |
| `get_result(challenge_id, learner) -> Option<Result>` | Read-only |
| `pause()` / `unpause()` | Emergency stop. Admin auth required |

The runner executes the tutor's tests against the learner's code in an isolated sandbox with no network access and fixed CPU, memory, and time limits. The result hash commits to the test suite hash, the solution hash, the toolchain version, and the output, so anyone with the same inputs can re-run and check it.

## MoltPod (study cohorts)

| Function | Purpose |
|---|---|
| `__constructor(PodConfig)` | Sets admin, token, finisher share, scholarship share, platform share, contest window, and the `CourseMarket` address |
| `create_pod(curator, goals_hash, stake, weeks, min_members, max_members, start_ledger) -> u64` | Creates a pod. The curator is usually a tutor. Returns `pod_id` |
| `join(learner, pod_id)` | Locks the exact stake and joins before the start ledger |
| `leave(learner, pod_id)` | Leaves before the start ledger for a full refund |
| `cancel_if_underfilled(pod_id)` | If `min_members` is not reached by the start ledger, the pod is cancelled and everyone is refunded. Publicly callable |
| `attest_progress(pod_id, learner, week, met)` | The curator or a runner records whether a learner met a week's goal. One attestation per (pod, learner, week) |
| `contest(pod_id, learner, week, evidence_hash)` | A learner contests an attestation within the contest window. An arbiter decides |
| `settle(pod_id)` | After the end ledger and contest window, computes finishers and forfeits. Publicly callable |
| `claim(pod_id, learner)` | A finisher claims their stake plus a share of forfeits |
| `get_pod(pod_id) -> Pod` / `get_member(pod_id, learner) -> Member` | Read-only |
| `pause()` / `unpause()` | Emergency stop. Admin auth required |

**Settlement.** A learner is a *finisher* if they met at least the required number of weekly goals (set at creation). Forfeited stakes from non-finishers are split: finishers (default 80%), scholarship balance (default 15%), and platform (default 5%). If everyone finishes, everyone simply gets their stake back. If nobody finishes, all forfeits go to the scholarship balance.

---

## Enforced Invariants and Test Mapping
Each invariant below is a design target and will be covered by the named tests.

**TutorRegistry**

- **Stake minimum**: a tutor cannot be active below the minimum stake (`InsufficientStake`). Test: `contracts/tutor-registry/src/test.rs -> test_register_below_min_stake_fails`
- **Verifier-only verification**: (`Unauthorized`). Test: `test_verify_requires_verifier`
- **Suspension blocks work**: a suspended tutor cannot list courses or accept sessions (`TutorSuspended`). Test: `test_suspended_tutor_cannot_create_course`, `test_suspended_tutor_cannot_accept_session`
- **Stake locked**: stake cannot be withdrawn during open sessions, disputes, or suspension (`StakeLocked`). Test: `test_withdraw_with_open_session_fails`
- **Ratings need a real purchase**: ratings come only from `CourseMarket` or `SessionEscrow`, once per (learner, item) (`Unauthorized`, `AlreadyRated`). Test: `test_rating_requires_market_caller`, `test_double_rating_fails`

**CourseMarket**

- **Versions are immutable**: a published content hash never changes. Test: `contracts/course-market/src/test.rs -> test_version_immutable`
- **Split sums to 100%**: tutor, platform, and scholarship shares total 10,000 bps (`InvalidSplit`). Test: `test_split_must_sum_to_10000`
- **Price changes are not retroactive**: existing purchases keep the price paid. Test: `test_price_change_does_not_affect_existing_purchase`
- **Payment is held**: payout cannot occur before the dispute window ends (`DisputeWindowOpen`). Test: `test_release_before_window_fails`, `test_release_after_window_succeeds`
- **Dispute freezes payout**: (`PurchaseDisputed`). Test: `test_release_while_disputed_fails`, `test_resolve_dispute_partial_refund`
- **Access is exact**: `has_access` is true only for paid or granted purchases and false after a full refund. Test: `test_has_access_after_purchase`, `test_no_access_after_full_refund`, `test_grant_access_requires_committee`
- **Delisted courses**: cannot be purchased, but existing access remains (`CourseDelisted`). Test: `test_purchase_delisted_course_fails`, `test_access_survives_delist`
- **Scholarship bound**: grants cannot exceed the scholarship balance (`InsufficientScholarship`). Test: `test_grant_above_balance_fails`
- **Replay protection and pause**: (`InvalidNonce`, `Paused`). Test: `test_replay_with_old_nonce_fails`, `test_purchase_when_paused_fails`

**SessionEscrow**

- **Funds locked at booking**: balance moves into escrow immediately. Test: `contracts/session-escrow/src/test.rs -> test_book_locks_funds`
- **Tutor-only accept and delivery**: (`Unauthorized`). Test: `test_accept_requires_tutor`, `test_mark_delivered_requires_tutor`
- **Cancellation rules**: a learner gets a full refund before the deadline and a partial refund after; a tutor cancel always refunds in full. Test: `test_early_cancel_full_refund`, `test_late_cancel_partial_refund`, `test_tutor_cancel_full_refund`
- **Auto-release only after timeout**: (`ConfirmTimeoutNotReached`). Test: `test_auto_release_before_timeout_fails`, `test_auto_release_after_timeout_succeeds`
- **No double settlement**: a session settles once (`AlreadySettled`). Test: `test_double_settle_fails`
- **No-show remedy**: a tutor no-show refunds the learner and flags the tutor's stake. Test: `test_no_show_refunds_learner`
- **Disputes freeze funds**: (`SessionDisputed`). Test: `test_confirm_while_disputed_fails`

**CompileVault**

- **One reward per learner per challenge**: (`AlreadyClaimed`). Test: `contracts/compile-vault/src/test.rs -> test_double_claim_fails`
- **Runner-only attestation**: only a registered runner can attest (`Unauthorized`). Test: `test_attest_requires_runner`, `test_removed_runner_cannot_attest`
- **Passing result required**: a failing attestation never pays (`NotPassed`). Test: `test_claim_with_failed_result_fails`
- **Solution matches submission**: the attested solution hash must equal the submitted one (`SolutionMismatch`). Test: `test_attest_with_wrong_solution_hash_fails`
- **No copied solutions**: the same solution hash cannot win for two different learners (`DuplicateSolution`). Test: `test_duplicate_solution_hash_fails`
- **Budget and winner caps**: payouts never exceed the budget or `max_winners` (`BudgetExhausted`, `MaxWinnersReached`). Test: `test_claim_above_budget_fails`, `test_claim_above_max_winners_fails`
- **Course-linked access**: a challenge linked to a course requires access (`NoCourseAccess`). Test: `test_course_linked_challenge_requires_access`
- **Deadline**: submissions and claims after the end ledger fail (`ChallengeEnded`). Test: `test_submit_after_end_fails`
- **Unspent budget returns**: only after the end ledger (`ChallengeNotEnded`). Test: `test_close_before_end_fails`, `test_close_after_end_refunds_author`
- **Plagiarism voids rewards**: (`Unauthorized` for non-arbiters). Test: `test_report_plagiarism_requires_arbiter`, `test_plagiarism_voids_reward`

**MoltPod**

- **Exact stake**: joining requires the exact stake amount (`WrongStake`). Test: `contracts/molt-pod/src/test.rs -> test_join_with_wrong_stake_fails`
- **Join and leave only before start**: (`PodStarted`). Test: `test_join_after_start_fails`, `test_leave_before_start_refunds`
- **Underfilled pods refund everyone**: (`PodNotUnderfilled` if it is filled). Test: `test_cancel_underfilled_refunds_all`, `test_cancel_filled_pod_fails`
- **Curator-only attestation, once per week**: (`Unauthorized`, `AlreadyAttested`). Test: `test_attest_requires_curator`, `test_double_attestation_fails`
- **Contest window**: learners can contest an attestation only within the window (`ContestWindowClosed`). Test: `test_contest_after_window_fails`, `test_contest_overturns_attestation`
- **Settle only after the end**: (`PodNotEnded`, `ContestWindowOpen`). Test: `test_settle_before_end_fails`, `test_settle_during_contest_window_fails`
- **Conservation**: total claims plus scholarship plus platform equals total stakes, with no creation or loss of funds. Test: `test_settlement_conserves_funds`, `test_no_finishers_all_to_scholarship`, `test_all_finish_all_refunded`
- **One claim each**: (`AlreadyClaimed`). Test: `test_double_claim_fails`

---

## Events

| Event | Topics | Data |
|---|---|---|
| `TutorRegisteredEvent` | `("tutor_registered_event", tutor)` | `stake`, `ledger` |
| `TutorVerifiedEvent` | `("tutor_verified_event", tutor)` | `verifier`, `ledger` |
| `CourseCreatedEvent` | `("course_created_event", course_id)` | `tutor`, `price`, `metadata_hash` |
| `CourseVersionEvent` | `("course_version_event", course_id)` | `version`, `content_hash` |
| `PurchaseEvent` | `("purchase_event", purchase_id)` | `course_id`, `buyer`, `price`, `release_ledger` |
| `PurchaseReleasedEvent` | `("purchase_released_event", purchase_id)` | `tutor_amount`, `platform_amount`, `scholarship_amount` |
| `AccessGrantedEvent` | `("access_granted_event", course_id)` | `learner`, `reason_hash` |
| `SessionBookedEvent` | `("session_booked_event", session_id)` | `learner`, `tutor`, `amount`, `start_ledger` |
| `SessionSettledEvent` | `("session_settled_event", session_id)` | `outcome`, `tutor_amount`, `learner_amount` |
| `ChallengeCreatedEvent` | `("challenge_created_event", challenge_id)` | `author`, `reward`, `budget`, `end_ledger` |
| `ResultAttestedEvent` | `("result_attested_event", challenge_id)` | `learner`, `passed`, `score_bps`, `result_hash` |
| `RewardClaimedEvent` | `("reward_claimed_event", challenge_id)` | `learner`, `amount` |
| `PodCreatedEvent` | `("pod_created_event", pod_id)` | `curator`, `stake`, `weeks`, `start_ledger` |
| `PodSettledEvent` | `("pod_settled_event", pod_id)` | `finishers`, `forfeited`, `scholarship_amount` |

Indexers should key course events on `course_id`, purchases on `purchase_id`, challenge events on `(challenge_id, learner)`, and pod events on `pod_id`. See `docs/EVENTS.md` for the pinned topic tuples.

## Error Codes
Every contract returns errors from a single shared enum in `contracts/common`. Codes are stable: new variants are appended with fresh values and never renumbered.

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Initialisation attempted twice |
| 2 | `NotInitialized` | Call made before initialisation |
| 3 | `Unauthorized` | Caller lacks the required authority |
| 4 | `Paused` | Contract is paused |
| 5 | `InvalidAmount` | Amount was not strictly positive |
| 6 | `InvalidNonce` | Replay-protection nonce did not match |
| 100 | `InsufficientStake` | Stake below the required minimum |
| 101 | `TutorSuspended` | Tutor is suspended |
| 102 | `StakeLocked` | Stake tied to open sessions or disputes |
| 103 | `AlreadyRated` | This purchase or session was already rated |
| 200 | `CourseNotFound` | No such course |
| 201 | `CourseDelisted` | Course is delisted |
| 202 | `InvalidSplit` | Split shares do not total 10,000 bps |
| 203 | `DisputeWindowOpen` | Release attempted inside the dispute window |
| 204 | `PurchaseDisputed` | Purchase is under dispute |
| 205 | `InsufficientScholarship` | Scholarship balance too low |
| 300 | `SessionNotFound` | No such session |
| 301 | `AlreadySettled` | Session already settled |
| 302 | `ConfirmTimeoutNotReached` | Auto-release attempted too early |
| 303 | `SessionDisputed` | Session is under dispute |
| 400 | `ChallengeNotFound` | No such challenge |
| 401 | `ChallengeEnded` | Challenge has ended |
| 402 | `ChallengeNotEnded` | Challenge has not ended |
| 403 | `NotPassed` | Result did not pass |
| 404 | `SolutionMismatch` | Attested solution differs from the submission |
| 405 | `DuplicateSolution` | Solution hash already used by another learner |
| 406 | `AlreadyClaimed` | Reward already claimed |
| 407 | `BudgetExhausted` | Challenge budget is spent |
| 408 | `MaxWinnersReached` | Winner cap reached |
| 409 | `NoCourseAccess` | Learner has no access to the linked course |
| 500 | `PodNotFound` | No such pod |
| 501 | `WrongStake` | Stake did not match the pod's stake |
| 502 | `PodStarted` | Action only allowed before the start ledger |
| 503 | `PodNotUnderfilled` | Pod meets its minimum members |
| 504 | `AlreadyAttested` | Week already attested for this learner |
| 505 | `ContestWindowClosed` | Contest window has ended |
| 506 | `ContestWindowOpen` | Settlement attempted inside the contest window |
| 507 | `PodNotEnded` | Pod has not ended |

## Storage and TTL
Soroban archives ledger entries that are not kept alive. Course listings, purchases (which prove access), and open pods and challenges use a TTL strategy that keeps active records in persistent storage for about 30 days (~518,400 ledgers) at a time. The backend's keeper service extends TTL for active records, and anyone can call the public `extend_*_ttl` functions. An archived entry can be restored with a restore transaction before it is read again.

## Amount Semantics
All amounts are integer token base units (`i128`). No floating-point arithmetic is used.

| Unit | Base units |
|---|---|
| 1 stroop (smallest) | 1 |
| 1 XLM | 10_000_000 |
| 5 XLM | 50_000_000 |

Percentages (splits, refunds, scores) are in basis points: 88% is `8_800`. Windows and deadlines are in Stellar ledgers, about 5 seconds each. For example, a 3-day dispute window is roughly 51,840 ledgers, and a 4-week pod is roughly 483,840 ledgers.

---

## Security and Honest Limits

| Risk | Mitigation |
|---|---|
| **Farming rewards with fake accounts** | Course-linked challenges need paid or granted access, one reward per learner, duplicate-solution rejection, plagiarism reports, and per-challenge budgets |
| **Cheating the test runner** | The sandbox has no network, fixed limits, a read-only filesystem, a vendored crate allowlist, and hidden tests (planned). Result hashes can be reproduced |
| **Compromised runner key** | Runner keys are rotated by admin multisig, and challenge budgets cap the damage |
| **Unfair pod attestations** | A contest window, an arbiter, and a record of every attestation |
| **Tutor fraud** | Stake, suspension, disputed-purchase freezes, and verified-purchase-only ratings |
| **Reentrancy** | State is updated before external transfers, and guarded entry points reject re-entry |

Known limits:
- Pods with forfeited stakes resemble commitment contracts, and the rules for them differ by country. Pod terms should get legal review before Mainnet, and operators can disable forfeit sharing if needed.
- The contract cannot tell whether a learner wrote the code themselves. The runner and arbiter reduce cheating, but cannot remove it.
- Trading and AI features live off-chain. Nothing in these contracts gives financial advice.

Full details are in `docs/SECURITY_MODEL.md`.

---

## Getting Started

### Prerequisites
```bash
rustup target add wasm32v1-none
cargo install --locked stellar-cli
```
A funded Stellar Testnet account: `stellar keys generate --fund`

### Build and test
```bash
git clone https://github.com/your-org/crabwise-contracts.git
cd crabwise-contracts
stellar contract build
cargo test
```

### Deploy to Testnet
```bash
bash scripts/deploy_testnet.sh
```

---

## Live on Testnet

| Contract | ID |
|---|---|
| TutorRegistry | *to be deployed* |
| CourseMarket | *to be deployed* |
| SessionEscrow | *to be deployed* |
| CompileVault | *to be deployed* |
| MoltPod | *to be deployed* |

---

## Roadmap

| Phase | Milestone |
|---|---|
| 1 | `TutorRegistry` and `CourseMarket` on Testnet |
| 2 | `SessionEscrow` and verified-purchase ratings |
| 3 | `CompileVault` (Proof of Compile) with a Testnet runner |
| 4 | `MoltPod` and the scholarship fund |
| 5 | Security audit, legal review of pods, Mainnet launch |

---

## Planned Issues for Contributors

| Type | Examples |
|---|---|
| **Tests** | One test per invariant above, split and rounding maths, cross-contract integration tests |
| **Bug fixes** | Edge cases around window boundaries, refund rounding, and TTL extension |
| **New features** | Referral splits, course bundles, subscription access, multi-token support |
| **Optimisation** | Storage layout, event size, and fee measurements per call |
| **Documentation** | Security model, events reference, deployment guide, worked examples for each contract |
| **Good first issues** | Typos, extra getters, doc comments, error message wording, example scripts |

Every issue has a description, acceptance criteria, and a pointer to the relevant files.

---

## Contributing

```bash
git checkout -b feat/your-feature
cargo test
cargo clippy -- -D warnings
# open a PR against main
```
See `CONTRIBUTING.md` and `docs/SECURITY_MODEL.md`.

## Security Disclosure
Responsible disclosure: `security@your-domain.example`

## License
MIT, 2026 Crabwise Contributors

---

*Built on Stellar. Powered by Soroban. Learn Rust, wisely.*
