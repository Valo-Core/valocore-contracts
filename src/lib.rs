#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MilestoneBounty {
    pub reward: i128,
    pub creator: Address,
    pub is_released: bool,
    pub review_deadline: u64,
    pub auto_claimed: bool,
}

#[contracttype]
pub enum StorageKey {
    Admin,
    TrackingTokenId,
    Milestone(u64),
    MilestoneCount,
    DefaultGracePeriod,
}

#[contract]
pub struct ValocoreEscrowContract;

#[contractimpl]
impl ValocoreEscrowContract {
    /// Initializes contract with admin, tracking token ID, and default grace period for reviews in seconds.
    pub fn initialize(
        env: Env,
        admin: Address,
        tracking_token_id: String,
        default_grace_period: u64,
    ) {
        if env.storage().instance().has(&StorageKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();

        env.storage().instance().set(&StorageKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&StorageKey::TrackingTokenId, &tracking_token_id);
        env.storage()
            .instance()
            .set(&StorageKey::MilestoneCount, &0u64);
        env.storage()
            .instance()
            .set(&StorageKey::DefaultGracePeriod, &default_grace_period);
    }

    /// Creates a new milestone bounty with the contract's default review grace period.
    pub fn create_milestone_bounty(env: Env, reward: i128) -> u64 {
        let default_grace: u64 = env
            .storage()
            .instance()
            .get(&StorageKey::DefaultGracePeriod)
            .unwrap_or(0);
        Self::create_milestone_with_grace(env, reward, default_grace)
    }

    /// Creates a new milestone bounty with a custom review grace period (in seconds).
    pub fn create_milestone_with_grace(env: Env, reward: i128, grace_period_secs: u64) -> u64 {
        let admin: Address = env.storage().instance().get(&StorageKey::Admin).unwrap();
        admin.require_auth();

        if reward <= 0 {
            panic!("Reward must be positive");
        }

        let count: u64 = env
            .storage()
            .instance()
            .get(&StorageKey::MilestoneCount)
            .unwrap();
        let new_count = count.checked_add(1).unwrap();

        let now = env.ledger().timestamp();
        let review_deadline = now
            .checked_add(grace_period_secs)
            .expect("Timestamp overflow");

        let bounty = MilestoneBounty {
            reward,
            creator: admin,
            is_released: false,
            review_deadline,
            auto_claimed: false,
        };

        env.storage()
            .instance()
            .set(&StorageKey::Milestone(count), &bounty);
        env.storage()
            .instance()
            .set(&StorageKey::MilestoneCount, &new_count);

        count
    }

    /// Standard release path invoked by admin/creator upon successful manual review.
    pub fn release_bounty(env: Env, milestone_id: u64, recipient: Address) {
        let admin: Address = env.storage().instance().get(&StorageKey::Admin).unwrap();
        admin.require_auth();

        let mut bounty: MilestoneBounty = env
            .storage()
            .instance()
            .get(&StorageKey::Milestone(milestone_id))
            .unwrap_or_else(|| panic!("Milestone not found"));

        if bounty.is_released {
            panic!("Bounty already released");
        }

        recipient.require_auth();

        bounty.is_released = true;
        env.storage()
            .instance()
            .set(&StorageKey::Milestone(milestone_id), &bounty);
    }

    /// Automated safety check allowing a contributor to auto-claim reward if maintainer is inactive past review deadline.
    pub fn auto_claim_bounty(env: Env, milestone_id: u64, recipient: Address) {
        recipient.require_auth();

        let mut bounty: MilestoneBounty = env
            .storage()
            .instance()
            .get(&StorageKey::Milestone(milestone_id))
            .unwrap_or_else(|| panic!("Milestone not found"));

        if bounty.is_released {
            panic!("Bounty already released");
        }

        let now = env.ledger().timestamp();
        if now < bounty.review_deadline {
            panic!("Review deadline has not passed");
        }

        bounty.is_released = true;
        bounty.auto_claimed = true;

        env.storage()
            .instance()
            .set(&StorageKey::Milestone(milestone_id), &bounty);
    }

    pub fn get_milestone(env: Env, milestone_id: u64) -> MilestoneBounty {
        env.storage()
            .instance()
            .get(&StorageKey::Milestone(milestone_id))
            .unwrap_or_else(|| panic!("Milestone not found"))
    }

    pub fn is_milestone_review_expired(env: Env, milestone_id: u64) -> bool {
        let bounty = Self::get_milestone(env.clone(), milestone_id);
        env.ledger().timestamp() >= bounty.review_deadline
    }

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&StorageKey::Admin).unwrap()
    }

    pub fn get_tracking_token_id(env: Env) -> String {
        env.storage()
            .instance()
            .get(&StorageKey::TrackingTokenId)
            .unwrap()
    }

    pub fn get_milestone_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&StorageKey::MilestoneCount)
            .unwrap()
    }

    pub fn get_default_grace_period(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&StorageKey::DefaultGracePeriod)
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, Ledger as _},
        Address, Env, String,
    };

    fn setup_contract(env: &Env, grace_period: u64) -> (ValocoreEscrowContractClient<'_>, Address) {
        let admin = Address::generate(env);
        let contract_id = env.register_contract(None, ValocoreEscrowContract);
        let client = ValocoreEscrowContractClient::new(env, &contract_id);
        let tracking_token = String::from_str(env, "TOKEN123");
        client.initialize(&admin, &tracking_token, &grace_period);
        (client, admin)
    }

    #[test]
    fn test_initialization_with_grace_period() {
        let env = Env::default();
        env.mock_all_auths();
        let (client, admin) = setup_contract(&env, 86400);

        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_default_grace_period(), 86400);
        assert_eq!(client.get_milestone_count(), 0);
    }

    #[test]
    fn test_create_milestone_with_default_grace_period() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000);
        let (client, admin) = setup_contract(&env, 7_200); // 2 hours

        let milestone_id = client.create_milestone_bounty(&500);
        assert_eq!(milestone_id, 0);

        let milestone = client.get_milestone(&milestone_id);
        assert_eq!(milestone.reward, 500);
        assert_eq!(milestone.creator, admin);
        assert!(!milestone.is_released);
        assert!(!milestone.auto_claimed);
        assert_eq!(milestone.review_deadline, 1_000 + 7_200);
        assert!(!client.is_milestone_review_expired(&milestone_id));
    }

    #[test]
    fn test_create_milestone_with_custom_grace_period() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(5_000);
        let (client, admin) = setup_contract(&env, 86400);

        // Custom grace period: 3,600s (1 hour)
        let milestone_id = client.create_milestone_with_grace(&1_000, &3_600);
        assert_eq!(milestone_id, 0);

        let milestone = client.get_milestone(&milestone_id);
        assert_eq!(milestone.reward, 1_000);
        assert_eq!(milestone.creator, admin);
        assert_eq!(milestone.review_deadline, 5_000 + 3_600);
    }

    #[test]
    fn test_manual_release_before_deadline_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(10_000);
        let (client, _admin) = setup_contract(&env, 3_600);

        let recipient = Address::generate(&env);
        let milestone_id = client.create_milestone_bounty(&250);

        // Advance 1,000s (deadline is at 13,600)
        env.ledger().set_timestamp(11_000);
        client.release_bounty(&milestone_id, &recipient);

        let milestone = client.get_milestone(&milestone_id);
        assert!(milestone.is_released);
        assert!(!milestone.auto_claimed);
    }

    #[test]
    fn test_auto_claim_after_deadline_succeeds() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(20_000);
        let (client, _admin) = setup_contract(&env, 4_000);

        let contributor = Address::generate(&env);
        let milestone_id = client.create_milestone_bounty(&800);

        // Before deadline: 22,000 (deadline is at 24,000)
        env.ledger().set_timestamp(22_000);
        assert!(!client.is_milestone_review_expired(&milestone_id));

        // Advance time past review deadline to 24,001
        env.ledger().set_timestamp(24_001);
        assert!(client.is_milestone_review_expired(&milestone_id));

        // Contributor signals auto-claim
        client.auto_claim_bounty(&milestone_id, &contributor);

        let milestone = client.get_milestone(&milestone_id);
        assert!(milestone.is_released);
        assert!(milestone.auto_claimed);
    }

    #[test]
    #[should_panic(expected = "Review deadline has not passed")]
    fn test_auto_claim_before_deadline_panics() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(10_000);
        let (client, _admin) = setup_contract(&env, 5_000);

        let contributor = Address::generate(&env);
        let milestone_id = client.create_milestone_bounty(&300);

        // Advance only 2,000s (deadline is at 15,000)
        env.ledger().set_timestamp(12_000);

        // Contributor tries to auto-claim before deadline - must fail
        client.auto_claim_bounty(&milestone_id, &contributor);
    }

    #[test]
    #[should_panic(expected = "Bounty already released")]
    fn test_auto_claim_already_released_panics() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(100);
        let (client, _admin) = setup_contract(&env, 100);

        let recipient = Address::generate(&env);
        let milestone_id = client.create_milestone_bounty(&100);

        // Manual release first
        client.release_bounty(&milestone_id, &recipient);

        // Advance past deadline
        env.ledger().set_timestamp(500);

        // Attempt to auto-claim must fail
        client.auto_claim_bounty(&milestone_id, &recipient);
    }
}
