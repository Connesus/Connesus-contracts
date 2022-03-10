use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalStatus {
    InProgress,
    /// If quorum voted yes, this proposal is successfully approved.
    Approved,
    /// If quorum voted no, this proposal is rejected. Bond is returned.
    Rejected,
    /// If quorum voted to remove (e.g. spam), this proposal is rejected and bond is not returned.
    /// Interfaces shouldn't show removed proposals.
    Removed,
    /// Expired after period of time.
    Expired,
    /// If proposal was moved to Hub or somewhere else.
    Moved,
    /// If proposal has failed when finalizing. Allowed to re-finalize again to either expire or approved.
    Failed,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionCall {
    method_name: String,
    args: Base64VecU8,
    deposit: U128,
    gas: U64,
}

/// Function call arguments.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PolicyParameters {
    pub proposal_bond: Option<U128>,
    pub proposal_period: Option<U64>,
    pub bounty_bond: Option<U128>,
    pub bounty_forgiveness_period: Option<U64>,
}

/// Kinds of proposals, doing different action.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Clone, Debug))]
#[serde(crate = "near_sdk::serde")]
pub enum ProposalKind {
    // Proposal for donations
    Donate,
    // Proposal for influencers to make a Crowdfunding
    // Funding {threshold: Balance, min_amount: Balance, max_approved_option: u8},
    // Change the DAO config.
    // Calls `receiver_id` with list of method names in a single promise.
    // Allows this contract to execute any arbitrary set of actions in other contracts.
    Vote {vote_kind: VoteKind},
}

impl ProposalKind {
    /// Returns label of policy for given type of proposal.
    pub fn to_policy_label(&self) -> &str {
        match self {
            ProposalKind::Donate=> "donate",
            ProposalKind::Vote {..} => "vote",
            // ProposalKind::Funding { threshold, min_amount, max_approved_option } => "funding",
        }
    }
}

/// Votes recorded in the proposal.
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
    pub total_vote_weight: Balance,
    pub vote: HashMap<AccountId, Balance>,
}

pub struct VoteOptionInput {
    pub title: String,
    pub description: String,
    pub min_vote_weight: Balance,
}

impl From<VoteOptionInput> for VoteOption {
    fn from(input: VoteOptionInput) -> Self {
        Self {
            title: input.title,
            description: input.description,
            min_vote_weight: input.min_vote_weight,
            total_vote_weight: 0,
            vote: HashMap::default()
        }
    }
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

/// Proposal that are sent to this DAO.
#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Proposal {
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,

    pub options: HashMap<String, VoteOption>,

    /// Submission time (for voting period).
    pub submission_time: U64,
    pub expired: U64,

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
        assert!(self.options.get(option_id).is_some(), "Invalid option id");
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
            _ => unreachable!()
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
            _ => unreachable!()
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
            _ => unreachable!()
        };
        
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ProposalInput {
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    pub expired: U64,
    pub options: HashMap<String, VoteOption>,
}

impl From<ProposalInput> for Proposal {
    fn from(input: ProposalInput) -> Self {
        assert!(input.expired.0 > env::block_timestamp(), "Expired must larger than current block timestamp");
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
                    expired: input.expired
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
                    expired: input.expired
                }
            },
        } 
        
    }
}

impl Contract {
    // Execute payout of given token to given user.
    // pub(crate) fn internal_payout(
    //     &mut self,
    //     token_id: &Option<AccountId>,
    //     receiver_id: &AccountId,
    //     amount: Balance,
    //     memo: String,
    //     msg: Option<String>,
    // ) -> PromiseOrValue<()> {
    //     if token_id.is_none() {
    //         Promise::new(receiver_id.clone()).transfer(amount).into()
    //     } else {
    //         if let Some(msg) = msg {
    //             ext_fungible_token::ft_transfer_call(
    //                 receiver_id.clone(),
    //                 U128(amount),
    //                 Some(memo),
    //                 msg,
    //                 token_id.as_ref().unwrap().clone(),
    //                 ONE_YOCTO_NEAR,
    //                 GAS_FOR_FT_TRANSFER,
    //             )
    //         } else {
    //             ext_fungible_token::ft_transfer(
    //                 receiver_id.clone(),
    //                 U128(amount),
    //                 Some(memo),
    //                 token_id.as_ref().unwrap().clone(),
    //                 ONE_YOCTO_NEAR,
    //                 GAS_FOR_FT_TRANSFER,
    //             )
    //         }
    //         .into()
    //     }
    // }

    // fn internal_return_bonds(&mut self, policy: &Policy, proposal: &Proposal) -> Promise {
    //     match &proposal.kind {
    //         ProposalKind::BountyDone { .. } => {
    //             self.locked_amount -= policy.bounty_bond.0;
    //             Promise::new(proposal.proposer.clone()).transfer(policy.bounty_bond.0);
    //         }
    //         _ => {}
    //     }

    //     self.locked_amount -= policy.proposal_bond.0;
    //     Promise::new(proposal.proposer.clone()).transfer(policy.proposal_bond.0)
    // }

    // /// Executes given proposal and updates the contract's state.
    // fn internal_execute_proposal(
    //     &mut self,
    //     policy: &Policy,
    //     proposal: &Proposal,
    //     proposal_id: u64,
    // ) -> PromiseOrValue<()> {
    //     let result = match &proposal.kind {
    //         ProposalKind::ChangeConfig { config } => {
    //             self.config.set(config);
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::ChangePolicy { policy } => {
    //             self.policy.set(policy);
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::AddMemberToRole { member_id, role } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.add_member_to_role(role, &member_id.clone().into());
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::RemoveMemberFromRole { member_id, role } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.remove_member_from_role(role, &member_id.clone().into());
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::FunctionCall {
    //             receiver_id,
    //             actions,
    //         } => {
    //             let mut promise = Promise::new(receiver_id.clone().into());
    //             for action in actions {
    //                 promise = promise.function_call(
    //                     action.method_name.clone().into(),
    //                     action.args.clone().into(),
    //                     action.deposit.0,
    //                     Gas(action.gas.0),
    //                 )
    //             }
    //             promise.into()
    //         }
    //         ProposalKind::UpgradeSelf { hash } => {
    //             upgrade_using_factory(hash.clone());
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::UpgradeRemote {
    //             receiver_id,
    //             method_name,
    //             hash,
    //         } => {
    //             upgrade_remote(&receiver_id, method_name, &CryptoHash::from(hash.clone()));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::Transfer {
    //             token_id,
    //             receiver_id,
    //             amount,
    //             msg,
    //         } => self.internal_payout(
    //             &convert_old_to_new_token(token_id),
    //             &receiver_id,
    //             amount.0,
    //             proposal.description.clone(),
    //             msg.clone(),
    //         ),
    //         ProposalKind::SetStakingContract { community_token_id } => {
    //             assert!(self.community_token_id.is_none(), "ERR_INVALID_STAKING_CHANGE");
    //             self.community_token_id = Some(community_token_id.clone().into());
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::AddBounty { bounty } => {
    //             self.internal_add_bounty(bounty);
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::BountyDone {
    //             bounty_id,
    //             receiver_id,
    //         } => self.internal_execute_bounty_payout(*bounty_id, &receiver_id.clone().into(), true),
    //         ProposalKind::Vote => PromiseOrValue::Value(()),
    //         ProposalKind::FactoryInfoUpdate { factory_info } => {
    //             internal_set_factory_info(factory_info);
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::ChangePolicyAddOrUpdateRole { role } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.add_or_update_role(role);
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::ChangePolicyRemoveRole { role } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.remove_role(role);
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::ChangePolicyUpdateDefaultVotePolicy { vote_policy } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.update_default_vote_policy(vote_policy);
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //         ProposalKind::ChangePolicyUpdateParameters { parameters } => {
    //             let mut new_policy = policy.clone();
    //             new_policy.update_parameters(parameters);
    //             self.policy.set(&VersionedPolicy::Current(new_policy));
    //             PromiseOrValue::Value(())
    //         }
    //     };
    //     match result {
    //         PromiseOrValue::Promise(promise) => promise
    //             .then(ext_self::on_proposal_callback(
    //                 proposal_id,
    //                 env::current_account_id(),
    //                 0,
    //                 GAS_FOR_FT_TRANSFER,
    //             ))
    //             .into(),
    //         PromiseOrValue::Value(()) => self.internal_return_bonds(&policy, &proposal).into(),
    //     }
    // }

    // pub(crate) fn internal_callback_proposal_success(
    //     &mut self,
    //     proposal: &mut Proposal,
    // ) -> PromiseOrValue<()> {
    //     let policy = self.policy.get().unwrap().to_policy();
    //     if let ProposalKind::BountyDone { bounty_id, .. } = proposal.kind {
    //         let mut bounty: Bounty = self.bounties.get(&bounty_id).expect("ERR_NO_BOUNTY").into();
    //         if bounty.times == 0 {
    //             self.bounties.remove(&bounty_id);
    //         } else {
    //             bounty.times -= 1;
    //             self.bounties
    //                 .insert(&bounty_id, &VersionedBounty::Default(bounty));
    //         }
    //     }
    //     proposal.status = ProposalStatus::Approved;
    //     self.internal_return_bonds(&policy, &proposal).into()
    // }

    // pub(crate) fn internal_callback_proposal_fail(
    //     &mut self,
    //     proposal: &mut Proposal,
    // ) -> PromiseOrValue<()> {
    //     proposal.status = ProposalStatus::Failed;
    //     PromiseOrValue::Value(())
    // }

    // /// Process rejecting proposal.
    // fn internal_reject_proposal(
    //     &mut self,
    //     policy: &Policy,
    //     proposal: &Proposal,
    //     return_bonds: bool,
    // ) -> PromiseOrValue<()> {
    //     if return_bonds {
    //         // Return bond to the proposer.
    //         self.internal_return_bonds(policy, proposal);
    //     }
    //     match &proposal.kind {
    //         ProposalKind::BountyDone {
    //             bounty_id,
    //             receiver_id,
    //         } => {
    //             self.internal_execute_bounty_payout(*bounty_id, &receiver_id.clone().into(), false)
    //         }
    //         _ => PromiseOrValue::Value(()),
    //     }
    // }

    // pub(crate) fn internal_user_info(&self) -> UserInfo {
    //     let account_id = env::predecessor_account_id();
    //     UserInfo {
    //         amount: self.get_user_weight(&account_id),
    //         account_id,
    //     }
    // }
}

#[near_bindgen]
impl Contract {
    // Add proposal to this DAO.
    // #[payable]
    // pub fn add_proposal(&mut self, proposal: ProposalInput) -> u64 {
    //     // 0. validate bond attached.
    //     // TODO: consider bond in the token of this DAO.
    //     let policy = self.policy.get().unwrap().to_policy();
    //     assert!(
    //         env::attached_deposit() >= policy.proposal_bond.0,
    //         "ERR_MIN_BOND"
    //     );

    //     // 1. Validate proposal.
    //     match &proposal.kind {
    //         ProposalKind::ChangePolicy { policy } => match policy {
    //             VersionedPolicy::Current(_) => {}
    //             _ => panic!("ERR_INVALID_POLICY"),
    //         },
    //         ProposalKind::Transfer { token_id, msg, .. } => {
    //             assert!(
    //                 !(token_id == OLD_BASE_TOKEN) || msg.is_none(),
    //                 "ERR_BASE_TOKEN_NO_MSG"
    //             );
    //         }
    //         ProposalKind::SetStakingContract { .. } => assert!(
    //             self.community_token_id.is_none(),
    //             "ERR_STAKING_CONTRACT_CANT_CHANGE"
    //         ),
    //         // TODO: add more verifications.
    //         _ => {}
    //     };

    //     // 2. Check permission of caller to add this type of proposal.
    //     assert!(
    //         policy
    //             .can_execute_action(
    //                 self.internal_user_info(),
    //                 &proposal.kind,
    //                 &Action::AddProposal
    //             )
    //             .1,
    //         "ERR_PERMISSION_DENIED"
    //     );

    //     // 3. Actually add proposal to the current list of proposals.
    //     let id = self.last_proposal_id;
    //     self.proposals
    //         .insert(&id, &VersionedProposal::Default(proposal.into()));
    //     self.last_proposal_id += 1;
    //     self.locked_amount += env::attached_deposit();
    //     id
    // }

    // // Act on given proposal by id, if permissions allow.
    // // Memo is logged but not stored in the state. Can be used to leave notes or explain the action.
    // pub fn act_proposal(&mut self, id: u64, action: Action, memo: Option<String>) {
    //     let mut proposal: Proposal = self.proposals.get(&id).expect("ERR_NO_PROPOSAL").into();
    //     let policy = self.policy.get().unwrap().to_policy();
    //     // Check permissions for the given action.
    //     let (roles, allowed) =
    //         policy.can_execute_action(self.internal_user_info(), &proposal.kind, &action);
    //     assert!(allowed, "ERR_PERMISSION_DENIED");
    //     let sender_id = env::predecessor_account_id();
    //     // Update proposal given action. Returns true if should be updated in storage.
    //     let update = match action {
    //         Action::AddProposal => env::panic_str("ERR_WRONG_ACTION"),
    //         Action::RemoveProposal => {
    //             self.proposals.remove(&id);
    //             false
    //         }
    //         Action::VoteApprove | Action::VoteReject | Action::VoteRemove => {
    //             assert!(
    //                 matches!(proposal.status, ProposalStatus::InProgress),
    //                 "ERR_PROPOSAL_NOT_READY_FOR_VOTE"
    //             );
    //             proposal.update_votes(
    //                 &sender_id,
    //                 &roles,
    //                 Vote::from(action),
    //                 &policy,
    //                 self.get_user_weight(&sender_id),
    //             );
    //             // Updates proposal status with new votes using the policy.
    //             proposal.status =
    //                 policy.proposal_status(&proposal, roles, self.total_delegation_amount);
    //             if proposal.status == ProposalStatus::Approved {
    //                 self.internal_execute_proposal(&policy, &proposal, id);
    //                 true
    //             } else if proposal.status == ProposalStatus::Removed {
    //                 self.internal_reject_proposal(&policy, &proposal, false);
    //                 self.proposals.remove(&id);
    //                 false
    //             } else if proposal.status == ProposalStatus::Rejected {
    //                 self.internal_reject_proposal(&policy, &proposal, true);
    //                 true
    //             } else {
    //                 // Still in progress or expired.
    //                 true
    //             }
    //         }
    //         // There are two cases when proposal must be finalized manually: expired or failed.
    //         // In case of failed, we just recompute the status and if it still approved, we re-execute the proposal.
    //         // In case of expired, we reject the proposal and return the bond.
    //         // Corner cases:
    //         //  - if proposal expired during the failed state - it will be marked as expired.
    //         //  - if the number of votes in the group has changed (new members has been added) -
    //         //      the proposal can loose it's approved state. In this case new proposal needs to be made, this one can only expire.
    //         Action::Finalize => {
    //             proposal.status = policy.proposal_status(
    //                 &proposal,
    //                 policy.roles.iter().map(|r| r.name.clone()).collect(),
    //                 self.total_delegation_amount,
    //             );
    //             match proposal.status {
    //                 ProposalStatus::Approved => {
    //                     self.internal_execute_proposal(&policy, &proposal, id);
    //                 }
    //                 ProposalStatus::Expired => {
    //                     self.internal_reject_proposal(&policy, &proposal, true);
    //                 }
    //                 _ => {
    //                     env::panic_str("ERR_PROPOSAL_NOT_EXPIRED_OR_FAILED");
    //                 }
    //             }
    //             true
    //         }
    //         Action::MoveToHub => false,
    //     };
    //     if update {
    //         self.proposals
    //             .insert(&id, &VersionedProposal::Default(proposal));
    //     }
    //     if let Some(memo) = memo {
    //         log!("Memo: {}", memo);
    //     }
    // }

    // // Receiving callback after the proposal has been finalized.
    // // If successful, returns bond money to the proposal originator.
    // // If the proposal execution failed (funds didn't transfer or function call failure),
    // // move proposal to "Failed" state.
    // #[private]
    // pub fn on_proposal_callback(&mut self, proposal_id: u64) -> PromiseOrValue<()> {
    //     let mut proposal: Proposal = self
    //         .proposals
    //         .get(&proposal_id)
    //         .expect("ERR_NO_PROPOSAL")
    //         .into();
    //     assert_eq!(
    //         env::promise_results_count(),
    //         1,
    //         "ERR_UNEXPECTED_CALLBACK_PROMISES"
    //     );
    //     let result = match env::promise_result(0) {
    //         PromiseResult::NotReady => unreachable!(),
    //         PromiseResult::Successful(_) => self.internal_callback_proposal_success(&mut proposal),
    //         PromiseResult::Failed => self.internal_callback_proposal_fail(&mut proposal),
    //     };
    //     self.proposals
    //         .insert(&proposal_id, &VersionedProposal::Default(proposal.into()));
    //     result
    // }
}
