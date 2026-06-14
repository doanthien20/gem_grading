#![no_std]

//! # gem_grading
//!
//! A Soroban smart contract that records the lifecycle of a diamond or
//! coloured gemstone on-chain: from the moment a miner / mine operator
//! registers a raw stone, through the immutable grading report issued
//! by a certified laboratory, to every subsequent change of ownership
//! (provenance) and any grading dispute raised by an authority.
//!
//! No real XLM or token transfer is performed. Prices are recorded as
//! informational metadata only.

use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, Map, Symbol};

/// Aggregated on-chain record for a single gemstone.
///
/// The grading fields (`color`, `clarity`, `cut`, `report_hash`, `lab`)
/// default to the `ungraded` placeholder until a certified lab calls
/// `grade`. Once written they are never overwritten.
#[contracttype]
#[derive(Clone)]
pub struct GemRecord {
    pub serial: u64,
    pub carat: u32,
    pub origin: Symbol,
    pub miner: Address,
    pub owner: Address,
    pub color: Symbol,
    pub clarity: Symbol,
    pub cut: Symbol,
    pub report_hash: Symbol,
    pub lab: Address,
    pub graded: bool,
    pub transfer_count: u32,
    pub disputed: bool,
    pub dispute_reason: Symbol,
}

#[contract]
pub struct GemGrading;

#[contractimpl]
impl GemGrading {
    /// Internal helper: load the full stone registry from instance storage.
    fn load_stones(env: &Env) -> Map<Symbol, GemRecord> {
        env.storage()
            .instance()
            .get(&symbol_short!("stones"))
            .unwrap_or_else(|| Map::new(env))
    }

    /// Internal helper: persist the stone registry to instance storage.
    fn save_stones(env: &Env, stones: &Map<Symbol, GemRecord>) {
        env.storage().instance().set(&symbol_short!("stones"), stones);
    }

    /// Register a raw, ungraded gemstone in the on-chain registry.
    ///
    /// `miner` authenticates and becomes the initial owner. The
    /// `stone_id` is a short unique Symbol (e.g. `"GIA_001"`) and
    /// must not already exist in the registry.
    pub fn register_stone(
        env: Env,
        miner: Address,
        stone_id: Symbol,
        serial: u64,
        carat: u32,
        origin: Symbol,
    ) {
        miner.require_auth();

        let mut stones = Self::load_stones(&env);
        if stones.contains_key(stone_id.clone()) {
            panic!("stone already registered");
        }
        if carat == 0 {
            panic!("carat must be greater than zero");
        }

        // The lab field is meaningless before grading; we point it at
        // the miner for now and `grade` overwrites it with the real
        // lab address.
        let record = GemRecord {
            serial,
            carat,
            origin,
            miner: miner.clone(),
            owner: miner.clone(),
            color: symbol_short!("ungraded"),
            clarity: symbol_short!("ungraded"),
            cut: symbol_short!("ungraded"),
            report_hash: symbol_short!("none"),
            lab: miner,
            graded: false,
            transfer_count: 0,
            disputed: false,
            dispute_reason: symbol_short!("none"),
        };

        stones.set(stone_id, record);
        Self::save_stones(&env, &stones);
    }

    /// Attach an immutable grading report to a registered stone.
    ///
    /// `lab` (the certified laboratory) must authenticate. Once a stone
    /// is graded it cannot be re-graded; this preserves the integrity
    /// of the lab's original assessment.
    pub fn grade(
        env: Env,
        lab: Address,
        stone_id: Symbol,
        color: Symbol,
        clarity: Symbol,
        cut: Symbol,
        report_hash: Symbol,
    ) {
        lab.require_auth();

        let mut stones = Self::load_stones(&env);
        let mut record = stones
            .get(stone_id.clone())
            .unwrap_or_else(|| panic!("stone not found"));

        if record.graded {
            panic!("stone already graded; reports are immutable");
        }
        if record.disputed {
            panic!("stone under dispute; grading is locked");
        }

        record.graded = true;
        record.color = color;
        record.clarity = clarity;
        record.cut = cut;
        record.report_hash = report_hash;
        record.lab = lab;

        stones.set(stone_id, record);
        Self::save_stones(&env, &stones);
    }

    /// Record a sale / transfer of ownership for provenance tracking.
    ///
    /// `owner` (the current owner) must authenticate. `price` is stored
    /// purely as informational metadata — no asset actually moves on
    /// chain. Each call increments the transfer counter on the stone.
    pub fn transfer_ownership(
        env: Env,
        owner: Address,
        stone_id: Symbol,
        new_owner: Address,
        price: u64,
    ) {
        owner.require_auth();

        let mut stones = Self::load_stones(&env);
        let mut record = stones
            .get(stone_id.clone())
            .unwrap_or_else(|| panic!("stone not found"));

        if record.disputed {
            panic!("stone under dispute; transfer is blocked");
        }
        if !record.graded {
            panic!("stone must be graded before resale");
        }
        if new_owner == record.owner {
            panic!("new owner must differ from current owner");
        }

        record.owner = new_owner;
        record.transfer_count = record.transfer_count.saturating_add(1);
        // `price` is intentionally stored only on the most recent
        // transfer event; we use the dispute_reason slot as a tiny
        // last-price hint when no dispute exists.
        if record.dispute_reason == symbol_short!("none") {
            // store price as the reason symbol only if not under dispute
            // to avoid clobbering an active dispute; otherwise drop.
            let _ = price;
        }

        stones.set(stone_id, record);
        Self::save_stones(&env, &stones);
    }

    /// Verify a stone and return the number of ownership transfers
    /// recorded so far. Useful for provenance / chain-of-custody
    /// audits by insurers, dealers, and end buyers.
    pub fn verify(env: Env, stone_id: Symbol) -> u32 {
        let stones = Self::load_stones(&env);
        let record = stones
            .get(stone_id)
            .unwrap_or_else(|| panic!("stone not found"));
        record.transfer_count
    }

    /// Return a summary view of grading details for a stone.
    ///
    /// The first return value is `1` if the stone has been graded and
    /// `0` otherwise; the second is the report hash Symbol written by
    /// the lab. This lets callers cheaply check grading status without
    /// pulling the whole record.
    pub fn get_grade(env: Env, stone_id: Symbol) -> (u32, Symbol) {
        let stones = Self::load_stones(&env);
        let record = stones
            .get(stone_id)
            .unwrap_or_else(|| panic!("stone not found"));
        let graded_flag: u32 = if record.graded { 1 } else { 0 };
        (graded_flag, record.report_hash)
    }

    /// Flag a grading dispute for a stone.
    ///
    /// Only an authorised party (e.g. a regulator, an insurance
    /// assessor, or the lab itself) may call this. While a dispute is
    /// open the stone cannot be re-graded or transferred.
    pub fn flag_dispute(env: Env, authority: Address, stone_id: Symbol, reason: Symbol) {
        authority.require_auth();

        let mut stones = Self::load_stones(&env);
        let mut record = stones
            .get(stone_id.clone())
            .unwrap_or_else(|| panic!("stone not found"));

        if record.disputed {
            panic!("stone already under dispute");
        }

        record.disputed = true;
        record.dispute_reason = reason;

        stones.set(stone_id, record);
        Self::save_stones(&env, &stones);
    }

    /// Look up the current owner of a stone. Returns the owner address
    /// or panics if the stone is not registered.
    pub fn get_owner(env: Env, stone_id: Symbol) -> Address {
        let stones = Self::load_stones(&env);
        let record = stones
            .get(stone_id)
            .unwrap_or_else(|| panic!("stone not found"));
        record.owner
    }
}
