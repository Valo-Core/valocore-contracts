#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneBounty {
    pub reward: i128,
    pub creator: Address,
    pub is_released: bool,
}

#[contracttype]
pub enum StorageKey {
    Admin,
    TrackingTokenId,
    Milestone(u64),
    MilestoneCount,
}

#[contract]
pub struct ValocoreEscrowContract;

#[contractimpl]
impl ValocoreEscrowContract {
    pub fn initialize(env: Env, admin: Address, tracking_token_id: String) {
        if env.storage().instance().has(&StorageKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&StorageKey::Admin, &admin);
        env.storage().instance().set(&StorageKey::TrackingTokenId, &tracking_token_id);
        env.storage().instance().set(&StorageKey::MilestoneCount, &0u64);
    }

    pub fn create_milestone_bounty(env: Env, reward: i128) -> u64 {
        let admin: Address = env.storage().instance().get(&StorageKey::Admin).unwrap();
        admin.require_auth();

        if reward <= 0 {
            panic!("Reward must be positive");
        }

        let mut count: u64 = env.storage().instance().get(&StorageKey::MilestoneCount).unwrap();
        let new_count = count.checked_add(1).unwrap();

        let bounty = MilestoneBounty {
            reward,
            creator: admin.clone(),
            is_released: false,
        };

        env.storage().instance().set(&StorageKey::Milestone(count), &bounty);
        env.storage().instance().set(&StorageKey::MilestoneCount, &new_count);

        count
    }

    pub fn release_bounty(env: Env, milestone_id: u64, recipient: Address) {
        let admin: Address = env.storage().instance().get(&StorageKey::Admin).unwrap();
        admin.require_auth();

        let mut bounty: MilestoneBounty = env.storage()
            .instance()
            .get(&StorageKey::Milestone(milestone_id))
            .unwrap_or_else(|| panic!("Milestone not found"));

        if bounty.is_released {
            panic!("Bounty already released");
        }

        recipient.require_auth();

        bounty.is_released = true;
        env.storage().instance().set(&StorageKey::Milestone(milestone_id), &bounty);
    }

    pub fn get_milestone(env: Env, milestone_id: u64) -> MilestoneBounty {
        env.storage()
            .instance()
            .get(&StorageKey::Milestone(milestone_id))
            .unwrap_or_else(|| panic!("Milestone not found"))
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&StorageKey::Admin).unwrap()
    }

    pub fn get_tracking_token_id(env: Env) -> String {
        env.storage().instance().get(&StorageKey::TrackingTokenId).unwrap()
    }

    pub fn get_milestone_count(env: Env) -> u64 {
        env.storage().instance().get(&StorageKey::MilestoneCount).unwrap()
    }
}
