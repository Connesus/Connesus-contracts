use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    Expired,
}

// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    Donate,
    Vote {vote_kind: VoteKind},
}

impl ProposalKind {
    // Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::Donate=> "donate",
            ProposalKind::Vote {..} => "vote",
            // ProposalKind::Funding { threshold, min_amount, max_approved_option } => "funding",
        }
    }
}

// Votes recorded in the proposal.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum VoteKind {
    // VoteByFunding,
    VoteByDelegation,
    MajorityVote
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct VoteOption {
    pub title: String,
    pub description: String,
    pub min_vote_weight: Balance,
}

impl From<VersionedProposal> for Proposal {
    fn from(v: VersionedProposal) -> Self {
        match v {
            VersionedProposal::Default(p) => p,
        }
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Vote {
    option: String,
    delegations: Balance
}

// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    // Original proposer.
    pub proposer: AccountId,
    // Description of this proposal.
    pub description: String,
    // Kind of proposal with relevant information.
    pub kind: ProposalKind,
    // Current status of the proposal.
    pub status: ProposalStatus,

    pub options: HashMap<String, VoteOption>,

    // Submission time (for voting period).
    pub submission_time: U64,
    pub duration: U64,

    pub donations: HashMap<AccountId, Balance>,
    pub total_donations: Balance,

    pub total_delegation_amount: Balance,
    pub votes: HashMap<AccountId, Vote>,
    pub option_delegations: HashMap<String, Balance>
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VersionedProposal {
    Default(Proposal),
}

impl Proposal {
    pub fn get_user_voted(&self, account_id: &AccountId) -> Option<&Vote> {
        self.votes.get(account_id)
    }

    // add total delegations, add vote, add vote delegations
    pub fn add_vote(&mut self, account_id: &AccountId, option_id: &String, vote_delegation: Balance, kind: &VoteKind) {
        assert!(self.options.get(option_id).is_some(), "INVALID_OPTION_ID");
        match kind {
            VoteKind::VoteByDelegation => {
                let vote = Vote {
                    option: option_id.to_string(),
                    delegations: vote_delegation
                };
                assert!(self.votes.insert(account_id.to_string(), vote).is_none(), "ERR_ALREADY_VOTED");
                self.total_delegation_amount += vote_delegation;
                let option_prev_delegation_amount = self.option_delegations.get(option_id).unwrap_or(&0);
                let option_new_delegation_amount = option_prev_delegation_amount + vote_delegation;
                self.option_delegations.insert(option_id.to_string(), option_new_delegation_amount);
            },
            VoteKind::MajorityVote => {
                let vote = Vote {
                    option: option_id.to_string(),
                    delegations: 1
                };
                assert!(self.votes.insert(account_id.to_string(), vote).is_none(), "ERR_ALREADY_VOTED");
                self.total_delegation_amount += 1;
                let option_prev_delegation_amount = self.option_delegations.get(option_id).unwrap_or(&0);
                let option_new_delegation_amount = option_prev_delegation_amount + 1;
                self.option_delegations.insert(option_id.to_string(), option_new_delegation_amount);
            },
        }
    }

    // remove total delegations, remove vote, remove vote delegations
    pub fn remove_vote(&mut self, account_id: &AccountId, kind: &VoteKind) {
        let vote = self.votes.remove(account_id).expect("ERR_DID_NOT_VOTED");
        match kind {
            VoteKind::VoteByDelegation => {
                self.total_delegation_amount -= vote.delegations;
                let option_prev_delegation_amount = self.option_delegations.get(&vote.option).unwrap_or(&0);
                let option_new_delegation_amount = option_prev_delegation_amount - vote.delegations;
                self.option_delegations.insert(vote.option, option_new_delegation_amount);
            },
            VoteKind::MajorityVote => {
                self.total_delegation_amount -= 1;
                let option_prev_delegation_amount = self.option_delegations.get(&vote.option).unwrap_or(&0);
                let option_new_delegation_amount = option_prev_delegation_amount - 1;
                self.option_delegations.insert(vote.option, option_new_delegation_amount);
            },
        }
    }

    pub fn donate(&mut self, account_id: &AccountId, amount: Balance) -> (Balance, Balance, Balance) {
        let prev_amount = self.donations.get(&account_id.to_string()).unwrap_or(&0).clone();
        let new_amount = 0 + amount;
        self.donations.insert(account_id.to_string(), new_amount);
        self.total_donations += amount;
        (prev_amount, new_amount, amount)
    } 

    // Adds vote of the given user with given `amount` of weight. If user already voted, fails.
    pub fn update_votes(
        &mut self,
        account_id: &AccountId,
        option_id: &String,
        delegation_amount: Balance 
    ) {
        let proposal_kind = self.kind.clone();
        match proposal_kind {
            ProposalKind::Vote { vote_kind } => {
                if self.votes.get(account_id).is_some() {
                    self.remove_vote(account_id, &vote_kind);
                };
                self.add_vote(account_id, option_id, delegation_amount, &vote_kind);
            },
            ProposalKind::Donate => {
                assert!(false, "Vote is not available for this proposal");
            },
        };
    }

    pub fn update_status(&mut self, status: ProposalStatus) {
        self.status = status;
    }

}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInput {
    // Description of this proposal.
    pub description: String,
    // Kind of proposal with relevant information.
    pub kind: ProposalKind,
    pub duration: U64,
    pub options: HashMap<String, VoteOption>,
}

impl From<ProposalInput> for Proposal {
    fn from(input: ProposalInput) -> Self {
        assert!(input.duration.0 > 1000000000 * 60 * 2, "DURATION_MIN_ERROR");
        match input.kind {
            ProposalKind::Donate => {
                Self {
                    proposer: env::predecessor_account_id(),
                    description: input.description,
                    kind: input.kind,
                    options: HashMap::default(),
                    status: ProposalStatus::InProgress,
                    submission_time: U64::from(env::block_timestamp()),
                    donations: HashMap::default(),
                    total_delegation_amount: 0,
                    total_donations: 0,
                    votes: HashMap::default(),
                    option_delegations: HashMap::default(),
                    duration: input.duration
                }
            }
            ProposalKind::Vote { .. } => {
                Self {
                    proposer: env::predecessor_account_id(),
                    description: input.description,
                    kind: input.kind,
                    options: input.options,
                    status: ProposalStatus::InProgress,
                    submission_time: U64::from(env::block_timestamp()),
                    donations: HashMap::default(),
                    total_delegation_amount: 0,
                    total_donations: 0,
                    votes: HashMap::default(),
                    option_delegations: HashMap::default(),
                    duration: input.duration
                }
            },
        } 
        
    }
}

#[near_bindgen]
impl Contract {
    // Add proposal to this DAO.
    pub fn add_proposal(&mut self, proposal_input: ProposalInput) -> u64 {

        // 1. Validate proposal.
        let  proposal = Proposal::from(proposal_input);

        // 3. Actually add proposal to the current list of proposals.
        let id = self.last_proposal_id;
        self.proposals
            .insert(&id, &VersionedProposal::Default(proposal.into()));
        self.last_proposal_id += 1;
        id
    }

    pub fn act_proposal(&mut self, id: u64, action: Action) {
        let account_id = env::predecessor_account_id();
        let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
        let user_delegate = self.delegations.get(&account_id).expect("ERR_NO_DELEGATION");
        {
            let proposal_end_time_stamp = proposal.submission_time.0 + proposal.duration.0;
            let current_block_timestamp = env::block_timestamp();
            assert!(proposal_end_time_stamp > current_block_timestamp, "PROPOSAL_EXPIRED");
        }
        match action {
            Action::Vote { option_id} => {
                match &proposal.kind.clone() {
                    ProposalKind::Vote { vote_kind } => {
                        proposal.add_vote(&account_id, &option_id, user_delegate, vote_kind);
                    },
                    _ => unreachable!()
                }
            },
            Action::Finalize => {
                assert_eq!(account_id, self.owner_id, "ONLY_OWNER");
                proposal.update_status(ProposalStatus::Expired);
            }
        }
    }
}
