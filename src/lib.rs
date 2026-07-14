use soroban_sdk::{contract, contractimpl, Address, Env, String, Map, Vec, panic_with_error};

// ===== Events =====

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockboxEvent {
    PingReceived = 0,
    BeneficiaryAdded = 1,
    BeneficiaryRemoved = 2,
    AssetsDeposited = 3,
    AssetWithdrawn = 4,
    InheritanceTriggered = 5,
    InheritanceClaimed = 6,
    ThresholdUpdated = 7,
}

// ===== Errors =====

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LockboxError {
    UnauthorizedAccess = 0,
    InvalidAllocation = 1,
    BeneficiaryNotFound = 2,
    AlreadyClaimed = 3,
    ThresholdNotExceeded = 4,
    InsufficientBalance = 5,
    InvalidThreshold = 6,
}

// ===== Data Structures =====

#[derive(Clone, Debug)]
pub struct BeneficiaryShare {
    pub address: Address,
    pub percentage: u32,
    pub claimed: bool,
    pub preferred_asset: String,
}

#[derive(Clone, Debug)]
pub struct LockboxState {
    pub owner: Address,
    pub last_ping_timestamp: u64,
    pub inactivity_threshold: u64,
    pub total_assets: i128,
    pub is_released: bool,
    pub release_timestamp: u64,
    pub encrypted_payload_cid: String,
    pub beneficiary_count: u32,
}

// ===== Main Contract =====

#[contract]
pub struct Lockbox;

#[contractimpl]
impl Lockbox {
    pub fn initialize(
        env: Env,
        owner: Address,
        inactivity_threshold: u64,
    ) -> Result<(), LockboxError> {
        owner.require_auth();

        let storage = env.storage().persistent();
        
        if storage.has(&String::from_slice(&env, b"owner")) {
            return Err(LockboxError::UnauthorizedAccess);
        }

        storage.set(&String::from_slice(&env, b"owner"), &owner);
        storage.set(&String::from_slice(&env, b"last_ping"), &env.ledger().timestamp());
        storage.set(&String::from_slice(&env, b"threshold"), &inactivity_threshold);
        storage.set(&String::from_slice(&env, b"total_assets"), &0i128);
        storage.set(&String::from_slice(&env, b"is_released"), &false);
        storage.set(&String::from_slice(&env, b"release_timestamp"), &0u64);
        storage.set(&String::from_slice(&env, b"encrypted_payload"), &String::from_slice(&env, b""));
        storage.set(&String::from_slice(&env, b"beneficiary_count"), &0u32);

        Ok(())
    }

    pub fn ping(env: Env) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        let current_time = env.ledger().timestamp();
        storage.set(&String::from_slice(&env, b"last_ping"), &current_time);

        env.events().publish(
            ("lockbox", "ping"),
            (owner.clone(), current_time),
        );

        Ok(())
    }

    pub fn add_beneficiary(
        env: Env,
        beneficiary: Address,
        percentage: u32,
        preferred_asset: String,
    ) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        if percentage > 100 {
            return Err(LockboxError::InvalidAllocation);
        }

        let key = String::from_slice(&env, format!("beneficiary_{}", beneficiary).as_bytes());
        let share = BeneficiaryShare {
            address: beneficiary.clone(),
            percentage,
            claimed: false,
            preferred_asset: preferred_asset.clone(),
        };
        storage.set(&key, &share);

        let count: u32 = storage.get(&String::from_slice(&env, b"beneficiary_count"))
            .unwrap_or(0);
        storage.set(&String::from_slice(&env, b"beneficiary_count"), &(count + 1));

        env.events().publish(
            ("lockbox", "beneficiary_added"),
            (owner, beneficiary, percentage),
        );

        Ok(())
    }

    pub fn deposit_assets(
        env: Env,
        amount: i128,
    ) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        if amount <= 0 {
            return Err(LockboxError::InvalidThreshold);
        }

        let current: i128 = storage.get(&String::from_slice(&env, b"total_assets"))
            .unwrap_or(0);
        storage.set(&String::from_slice(&env, b"total_assets"), &(current + amount));

        env.events().publish(
            ("lockbox", "deposit"),
            (owner, amount),
        );

        Ok(())
    }

    pub fn trigger_release(env: Env) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let is_released: bool = storage.get(&String::from_slice(&env, b"is_released"))
            .unwrap_or(false);
        if is_released {
            return Err(LockboxError::UnauthorizedAccess);
        }

        let last_ping: u64 = storage.get(&String::from_slice(&env, b"last_ping"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        let threshold: u64 = storage.get(&String::from_slice(&env, b"threshold"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        let current_time = env.ledger().timestamp();

        if current_time <= last_ping + threshold {
            return Err(LockboxError::ThresholdNotExceeded);
        }

        storage.set(&String::from_slice(&env, b"is_released"), &true);
        storage.set(&String::from_slice(&env, b"release_timestamp"), &current_time);

        env.events().publish(
            ("lockbox", "inheritance_triggered"),
            current_time,
        );

        Ok(())
    }

    pub fn claim_inheritance(
        env: Env,
        beneficiary: Address,
    ) -> Result<i128, LockboxError> {
        let storage = env.storage().persistent();
        beneficiary.require_auth();

        let is_released: bool = storage.get(&String::from_slice(&env, b"is_released"))
            .unwrap_or(false);
        if !is_released {
            return Err(LockboxError::ThresholdNotExceeded);
        }

        let key = String::from_slice(&env, format!("beneficiary_{}", beneficiary).as_bytes());
        let mut share: BeneficiaryShare = storage.get(&key)
            .ok_or(LockboxError::BeneficiaryNotFound)?;

        if share.claimed {
            return Err(LockboxError::AlreadyClaimed);
        }

        let total: i128 = storage.get(&String::from_slice(&env, b"total_assets"))
            .unwrap_or(0);
        let payout = (total * share.percentage as i128) / 100;

        share.claimed = true;
        storage.set(&key, &share);

        env.events().publish(
            ("lockbox", "inheritance_claimed"),
            (beneficiary.clone(), payout),
        );

        Ok(payout)
    }

    pub fn get_state(env: Env) -> LockboxState {
        let storage = env.storage().persistent();
        
        LockboxState {
            owner: storage.get(&String::from_slice(&env, b"owner"))
                .unwrap_or_else(|| Address::from_contract_id(&env, &env.current_contract_address())),
            last_ping_timestamp: storage.get(&String::from_slice(&env, b"last_ping"))
                .unwrap_or(0),
            inactivity_threshold: storage.get(&String::from_slice(&env, b"threshold"))
                .unwrap_or(0),
            total_assets: storage.get(&String::from_slice(&env, b"total_assets"))
                .unwrap_or(0),
            is_released: storage.get(&String::from_slice(&env, b"is_released"))
                .unwrap_or(false),
            release_timestamp: storage.get(&String::from_slice(&env, b"release_timestamp"))
                .unwrap_or(0),
            encrypted_payload_cid: storage.get(&String::from_slice(&env, b"encrypted_payload"))
                .unwrap_or_else(|| String::from_slice(&env, b"")),
            beneficiary_count: storage.get(&String::from_slice(&env, b"beneficiary_count"))
                .unwrap_or(0),
        }
    }

    pub fn get_days_until_trigger(env: Env) -> i64 {
        let storage = env.storage().persistent();
        
        let last_ping: u64 = storage.get(&String::from_slice(&env, b"last_ping"))
            .unwrap_or(0);
        let threshold: u64 = storage.get(&String::from_slice(&env, b"threshold"))
            .unwrap_or(0);
        let current_time = env.ledger().timestamp();
        
        let release_time = last_ping + threshold;
        if current_time >= release_time {
            -((current_time - release_time) / 86400) as i64
        } else {
            ((release_time - current_time) / 86400) as i64
        }
    }

    pub fn set_encrypted_payload(
        env: Env,
        cid: String,
    ) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        storage.set(&String::from_slice(&env, b"encrypted_payload"), &cid);
        Ok(())
    }

    pub fn set_inactivity_threshold(
        env: Env,
        new_threshold: u64,
    ) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        if new_threshold < 86400 {
            return Err(LockboxError::InvalidThreshold);
        }

        storage.set(&String::from_slice(&env, b"threshold"), &new_threshold);
        
        env.events().publish(
            ("lockbox", "threshold_updated"),
            new_threshold,
        );

        Ok(())
    }

    pub fn withdraw_assets(
        env: Env,
        amount: i128,
    ) -> Result<(), LockboxError> {
        let storage = env.storage().persistent();
        
        let owner: Address = storage.get(&String::from_slice(&env, b"owner"))
            .ok_or(LockboxError::UnauthorizedAccess)?;
        owner.require_auth();

        let is_released: bool = storage.get(&String::from_slice(&env, b"is_released"))
            .unwrap_or(false);
        if is_released {
            return Err(LockboxError::UnauthorizedAccess);
        }

        let current: i128 = storage.get(&String::from_slice(&env, b"total_assets"))
            .unwrap_or(0);
        if amount > current {
            return Err(LockboxError::InsufficientBalance);
        }

        storage.set(&String::from_slice(&env, b"total_assets"), &(current - amount));

        env.events().publish(
            ("lockbox", "withdrawal"),
            (owner, amount),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_placeholder() {
        assert_eq!(1, 1);
    }
}
