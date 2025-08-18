# Dynamic Turn-Taking Algorithms

## Overview

Turn-taking is fundamental to multi-party conversations. In distributed agent systems, sophisticated algorithms are needed to manage who speaks when, handle interruptions gracefully, and ensure fair participation. This document details various turn-taking algorithms, from simple queue-based approaches to complex bidding and consensus mechanisms.

## Core Turn-Taking Models

### Sequential Turn-Taking

The simplest model - one speaker at a time:

```rust
pub struct SequentialTurnManager {
    /// Current speaker
    current_speaker: Arc<RwLock<Option<ParticipantId>>>,
    
    /// Queue of waiting speakers
    queue: Arc<RwLock<VecDeque<TurnRequest>>>,
    
    /// Turn duration limits
    turn_limits: TurnLimits,
    
    /// Turn timer
    timer: Arc<TurnTimer>,
}

pub struct TurnLimits {
    /// Minimum time guaranteed
    min_duration: Duration,
    
    /// Maximum time allowed
    max_duration: Duration,
    
    /// Warning before timeout
    warning_threshold: Duration,
}

impl SequentialTurnManager {
    pub async fn request_turn(&self, request: TurnRequest) -> Result<TurnToken> {
        // Check if requester already has turn
        if let Some(current) = self.current_speaker.read().await.as_ref() {
            if current == &request.participant_id {
                return Err(Error::AlreadyHasTurn);
            }
        }
        
        // Add to queue
        let mut queue = self.queue.write().await;
        let position = queue.len();
        queue.push_back(request.clone());
        
        // Create token for tracking
        let token = TurnToken {
            id: TokenId::new(),
            participant: request.participant_id,
            queue_position: position,
            estimated_wait: self.estimate_wait_time(position).await?,
        };
        
        Ok(token)
    }
    
    pub async fn end_turn(&self, participant: ParticipantId) -> Result<()> {
        let mut current = self.current_speaker.write().await;
        
        if current.as_ref() != Some(&participant) {
            return Err(Error::NotCurrentSpeaker);
        }
        
        // Clear current speaker
        *current = None;
        
        // Grant turn to next in queue
        self.grant_next_turn().await?;
        
        Ok(())
    }
    
    async fn grant_next_turn(&self) -> Result<()> {
        let mut queue = self.queue.write().await;
        
        if let Some(next_request) = queue.pop_front() {
            // Set as current speaker
            *self.current_speaker.write().await = Some(next_request.participant_id.clone());
            
            // Start turn timer
            self.timer.start_turn(
                next_request.participant_id.clone(),
                self.turn_limits.max_duration,
            ).await?;
            
            // Notify participant
            self.notify_turn_granted(next_request.participant_id).await?;
        }
        
        Ok(())
    }
}
```

### Priority-Based Turn-Taking

Speakers have different priorities:

```rust
pub struct PriorityTurnManager {
    /// Priority calculator
    priority_calc: Arc<dyn PriorityCalculator>,
    
    /// Priority queue of requests
    queue: Arc<RwLock<BinaryHeap<PrioritizedRequest>>>,
    
    /// Preemption settings
    preemption: PreemptionSettings,
}

#[derive(Eq, PartialEq)]
pub struct PrioritizedRequest {
    request: TurnRequest,
    priority: Priority,
    timestamp: Instant,
}

impl Ord for PrioritizedRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then earlier timestamp
        self.priority.cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

pub struct PreemptionSettings {
    /// Allow higher priority to interrupt
    allow_preemption: bool,
    
    /// Minimum priority difference for preemption
    preemption_threshold: f64,
    
    /// Grace period before preemption
    grace_period: Duration,
}

pub trait PriorityCalculator: Send + Sync {
    fn calculate(&self, participant: &ParticipantId, context: &Context) -> Priority;
}

pub struct DynamicPriorityCalculator {
    /// Base priorities by role
    role_priorities: HashMap<Role, f64>,
    
    /// Factors affecting priority
    factors: Vec<Box<dyn PriorityFactor>>,
}

pub trait PriorityFactor: Send + Sync {
    fn adjust_priority(&self, base: f64, participant: &ParticipantId, context: &Context) -> f64;
}

impl PriorityTurnManager {
    pub async fn request_turn_with_preemption(
        &self,
        request: TurnRequest,
    ) -> Result<TurnResponse> {
        let priority = self.priority_calc.calculate(&request.participant_id, &request.context);
        
        // Check if can preempt current speaker
        if self.preemption.allow_preemption {
            if let Some(current) = self.get_current_speaker().await {
                let current_priority = self.get_speaker_priority(&current).await?;
                
                if priority.value > current_priority.value + self.preemption.preemption_threshold {
                    // Preempt after grace period
                    tokio::time::sleep(self.preemption.grace_period).await;
                    
                    self.preempt_speaker(request.participant_id).await?;
                    return Ok(TurnResponse::Granted { preempted: true });
                }
            }
        }
        
        // Add to priority queue
        let prioritized = PrioritizedRequest {
            request,
            priority,
            timestamp: Instant::now(),
        };
        
        self.queue.write().await.push(prioritized);
        
        Ok(TurnResponse::Queued { priority })
    }
}
```

## Bidding Mechanisms

### Auction-Based Turn Allocation

Participants bid for speaking rights:

```rust
pub struct AuctionTurnManager {
    /// Current auction
    current_auction: Arc<RwLock<Option<Auction>>>,
    
    /// Auction configuration
    config: AuctionConfig,
    
    /// Bid evaluator
    evaluator: Arc<dyn BidEvaluator>,
    
    /// Credit system
    credit_system: Arc<CreditSystem>,
}

pub struct Auction {
    /// Auction identifier
    id: AuctionId,
    
    /// Start and end times
    start_time: Instant,
    end_time: Instant,
    
    /// Submitted bids
    bids: Vec<Bid>,
    
    /// Auction type
    auction_type: AuctionType,
}

pub enum AuctionType {
    /// Highest bidder wins
    FirstPrice,
    
    /// Winner pays second-highest bid
    SecondPrice,
    
    /// All-pay auction
    AllPay,
    
    /// Combinatorial for multiple slots
    Combinatorial {
        slots: usize,
    },
}

pub struct Bid {
    /// Bidding participant
    participant: ParticipantId,
    
    /// Bid amount
    amount: BidAmount,
    
    /// Urgency factor
    urgency: f64,
    
    /// Estimated speaking duration
    duration: Duration,
    
    /// Content preview for evaluation
    preview: Option<ContentPreview>,
}

pub enum BidAmount {
    /// Virtual credits
    Credits(u64),
    
    /// Priority points
    Priority(f64),
    
    /// Reputation stake
    Reputation(f64),
    
    /// Composite bid
    Composite {
        credits: u64,
        priority: f64,
        reputation: f64,
    },
}

impl AuctionTurnManager {
    pub async fn start_auction(&self, duration: Duration) -> Result<AuctionId> {
        let auction = Auction {
            id: AuctionId::new(),
            start_time: Instant::now(),
            end_time: Instant::now() + duration,
            bids: Vec::new(),
            auction_type: self.config.auction_type.clone(),
        };
        
        let auction_id = auction.id.clone();
        *self.current_auction.write().await = Some(auction);
        
        // Notify all participants
        self.broadcast_auction_start(auction_id.clone(), duration).await?;
        
        // Schedule auction close
        let manager = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            manager.close_auction(auction_id).await;
        });
        
        Ok(auction_id)
    }
    
    pub async fn submit_bid(&self, bid: Bid) -> Result<BidReceipt> {
        let mut auction_guard = self.current_auction.write().await;
        
        if let Some(ref mut auction) = *auction_guard {
            if Instant::now() > auction.end_time {
                return Err(Error::AuctionClosed);
            }
            
            // Validate bid
            self.validate_bid(&bid, auction).await?;
            
            // Deduct bid amount (for all-pay auctions)
            if matches!(auction.auction_type, AuctionType::AllPay) {
                self.credit_system.deduct(&bid.participant, &bid.amount).await?;
            }
            
            auction.bids.push(bid.clone());
            
            Ok(BidReceipt {
                auction_id: auction.id.clone(),
                bid_id: BidId::new(),
                timestamp: Instant::now(),
            })
        } else {
            Err(Error::NoActiveAuction)
        }
    }
    
    async fn close_auction(&self, auction_id: AuctionId) -> Result<()> {
        let auction = self.current_auction.write().await.take();
        
        if let Some(auction) = auction {
            if auction.id != auction_id {
                return Err(Error::AuctionMismatch);
            }
            
            // Evaluate bids and determine winner(s)
            let winners = self.evaluator.evaluate(&auction).await?;
            
            // Process payments
            self.process_auction_payments(&auction, &winners).await?;
            
            // Grant turns to winners
            for winner in winners {
                self.grant_turn(winner.participant).await?;
            }
            
            // Start next auction if configured
            if self.config.continuous_auctions {
                self.start_auction(self.config.auction_duration).await?;
            }
        }
        
        Ok(())
    }
}
```

### Multi-Attribute Bidding

Bids evaluated on multiple criteria:

```rust
pub struct MultiAttributeBidEvaluator {
    /// Weights for different attributes
    weights: AttributeWeights,
    
    /// Scoring functions
    scorers: HashMap<Attribute, Box<dyn AttributeScorer>>,
}

pub struct AttributeWeights {
    pub urgency: f64,
    pub relevance: f64,
    pub quality: f64,
    pub fairness: f64,
    pub cost: f64,
}

pub trait AttributeScorer: Send + Sync {
    fn score(&self, bid: &Bid, context: &Context) -> f64;
}

pub struct UrgencyScorer {
    /// Decay function for urgency
    decay: DecayFunction,
}

pub struct RelevanceScorer {
    /// Topic model for relevance
    topic_model: Arc<TopicModel>,
    
    /// Current conversation context
    context: Arc<ConversationContext>,
}

pub struct FairnessScorer {
    /// Speaking time history
    history: Arc<SpeakingHistory>,
    
    /// Fairness metric
    metric: FairnessMetric,
}

impl MultiAttributeBidEvaluator {
    pub async fn evaluate(&self, auction: &Auction) -> Result<Vec<Winner>> {
        let mut scored_bids = Vec::new();
        
        for bid in &auction.bids {
            let mut total_score = 0.0;
            
            // Score each attribute
            for (attribute, scorer) in &self.scorers {
                let score = scorer.score(bid, &auction.context);
                let weight = self.weights.get(attribute);
                total_score += score * weight;
            }
            
            scored_bids.push(ScoredBid {
                bid: bid.clone(),
                score: total_score,
            });
        }
        
        // Sort by score
        scored_bids.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        
        // Select winners based on auction type
        match auction.auction_type {
            AuctionType::FirstPrice => {
                Ok(vec![Winner {
                    participant: scored_bids[0].bid.participant.clone(),
                    payment: scored_bids[0].bid.amount.clone(),
                }])
            }
            AuctionType::SecondPrice => {
                let winner = &scored_bids[0];
                let second_price = if scored_bids.len() > 1 {
                    scored_bids[1].bid.amount.clone()
                } else {
                    BidAmount::Credits(0)
                };
                
                Ok(vec![Winner {
                    participant: winner.bid.participant.clone(),
                    payment: second_price,
                }])
            }
            AuctionType::Combinatorial { slots } => {
                let winners = scored_bids
                    .iter()
                    .take(slots)
                    .map(|sb| Winner {
                        participant: sb.bid.participant.clone(),
                        payment: sb.bid.amount.clone(),
                    })
                    .collect();
                Ok(winners)
            }
            _ => Ok(vec![])
        }
    }
}
```

## Consensus-Based Allocation

### Voting Mechanisms

Participants vote on who should speak:

```rust
pub struct VotingTurnManager {
    /// Active voting session
    current_vote: Arc<RwLock<Option<VotingSession>>>,
    
    /// Voting rules
    rules: VotingRules,
    
    /// Vote counter
    counter: Arc<VoteCounter>,
}

pub struct VotingSession {
    /// Session ID
    id: SessionId,
    
    /// Candidates requesting to speak
    candidates: Vec<Candidate>,
    
    /// Collected votes
    votes: HashMap<ParticipantId, Vote>,
    
    /// Voting deadline
    deadline: Instant,
    
    /// Voting method
    method: VotingMethod,
}

pub enum VotingMethod {
    /// Simple majority
    SimpleMajority,
    
    /// Ranked choice
    RankedChoice,
    
    /// Approval voting
    Approval {
        max_approvals: usize,
    },
    
    /// Score voting
    Score {
        min_score: i32,
        max_score: i32,
    },
    
    /// Quadratic voting
    Quadratic {
        vote_budget: u32,
    },
}

pub struct Vote {
    voter: ParticipantId,
    timestamp: Instant,
    choice: VoteChoice,
}

pub enum VoteChoice {
    /// Single candidate
    Single(CandidateId),
    
    /// Ranked preferences
    Ranked(Vec<CandidateId>),
    
    /// Multiple approvals
    Approval(HashSet<CandidateId>),
    
    /// Scores for candidates
    Scores(HashMap<CandidateId, i32>),
    
    /// Quadratic vote distribution
    Quadratic(HashMap<CandidateId, u32>),
}

impl VotingTurnManager {
    pub async fn initiate_vote(&self, candidates: Vec<Candidate>) -> Result<SessionId> {
        let session = VotingSession {
            id: SessionId::new(),
            candidates,
            votes: HashMap::new(),
            deadline: Instant::now() + self.rules.voting_duration,
            method: self.rules.voting_method.clone(),
        };
        
        let session_id = session.id.clone();
        *self.current_vote.write().await = Some(session);
        
        // Notify participants
        self.broadcast_vote_request(session_id.clone()).await?;
        
        // Schedule vote closing
        let manager = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(self.rules.voting_duration).await;
            manager.close_voting(session_id).await;
        });
        
        Ok(session_id)
    }
    
    pub async fn cast_vote(&self, vote: Vote) -> Result<VoteReceipt> {
        let mut session_guard = self.current_vote.write().await;
        
        if let Some(ref mut session) = *session_guard {
            // Validate vote
            self.validate_vote(&vote, session)?;
            
            // Record vote (overwrite if already voted)
            session.votes.insert(vote.voter.clone(), vote);
            
            Ok(VoteReceipt {
                session_id: session.id.clone(),
                timestamp: Instant::now(),
            })
        } else {
            Err(Error::NoActiveVote)
        }
    }
    
    async fn close_voting(&self, session_id: SessionId) -> Result<()> {
        let session = self.current_vote.write().await.take();
        
        if let Some(session) = session {
            // Count votes
            let results = self.counter.count_votes(&session).await?;
            
            // Determine winner
            let winner = self.determine_winner(results, &session.method)?;
            
            // Grant turn to winner
            if let Some(winner_id) = winner {
                self.grant_turn(winner_id).await?;
            }
            
            // Notify results
            self.broadcast_results(results).await?;
        }
        
        Ok(())
    }
}
```

### Consensus Protocols

Achieving agreement on speaker selection:

```rust
pub struct ConsensusProtocolManager {
    /// Consensus algorithm
    protocol: ConsensusProtocol,
    
    /// Participant registry
    participants: Arc<RwLock<HashMap<ParticipantId, ParticipantInfo>>>,
    
    /// Current consensus round
    current_round: Arc<RwLock<Option<ConsensusRound>>>,
}

pub enum ConsensusProtocol {
    /// Byzantine Fault Tolerant
    BFT {
        fault_tolerance: usize,
    },
    
    /// Raft consensus
    Raft {
        leader: Arc<RwLock<Option<ParticipantId>>>,
        term: Arc<AtomicU64>,
    },
    
    /// Paxos
    Paxos {
        proposers: HashSet<ParticipantId>,
        acceptors: HashSet<ParticipantId>,
    },
    
    /// Proof of Stake
    ProofOfStake {
        stakes: HashMap<ParticipantId, u64>,
    },
}

pub struct ConsensusRound {
    round_number: u64,
    proposals: Vec<TurnProposal>,
    votes: HashMap<ParticipantId, ProposalVote>,
    phase: ConsensusPhase,
}

pub enum ConsensusPhase {
    Proposal,
    Voting,
    Commit,
    Finalized,
}

impl ConsensusProtocolManager {
    pub async fn propose_speaker(&self, proposal: TurnProposal) -> Result<ProposalId> {
        match &self.protocol {
            ConsensusProtocol::BFT { fault_tolerance } => {
                self.bft_propose(proposal, *fault_tolerance).await
            }
            
            ConsensusProtocol::Raft { leader, term } => {
                self.raft_propose(proposal, leader.clone(), term.clone()).await
            }
            
            ConsensusProtocol::ProofOfStake { stakes } => {
                self.pos_propose(proposal, stakes).await
            }
            
            _ => Err(Error::ProtocolNotSupported)
        }
    }
    
    async fn bft_propose(
        &self,
        proposal: TurnProposal,
        fault_tolerance: usize,
    ) -> Result<ProposalId> {
        let participants = self.participants.read().await;
        let total = participants.len();
        let required_votes = total - fault_tolerance;
        
        // Start new round
        let round = ConsensusRound {
            round_number: self.next_round_number().await,
            proposals: vec![proposal.clone()],
            votes: HashMap::new(),
            phase: ConsensusPhase::Proposal,
        };
        
        *self.current_round.write().await = Some(round);
        
        // Broadcast proposal
        self.broadcast_proposal(proposal.clone()).await?;
        
        // Collect votes
        let votes = self.collect_votes(required_votes).await?;
        
        // Check consensus
        if votes.len() >= required_votes {
            self.finalize_consensus(proposal.participant).await?;
            Ok(proposal.id)
        } else {
            Err(Error::ConsensusNotReached)
        }
    }
}
```

## Advanced Algorithms

### Reinforcement Learning-Based Allocation

ML-driven turn allocation:

```rust
pub struct RLTurnAllocator {
    /// RL model
    model: Arc<dyn TurnAllocationModel>,
    
    /// Experience buffer
    experience_buffer: Arc<ExperienceBuffer>,
    
    /// Training configuration
    training_config: TrainingConfig,
}

pub trait TurnAllocationModel: Send + Sync {
    fn predict(&self, state: &ConversationState) -> TurnAllocation;
    fn update(&mut self, experience: Experience);
}

pub struct ConversationState {
    /// Current participants
    participants: Vec<ParticipantFeatures>,
    
    /// Conversation history
    history: ConversationFeatures,
    
    /// Pending requests
    requests: Vec<RequestFeatures>,
    
    /// Context features
    context: ContextFeatures,
}

pub struct Experience {
    state: ConversationState,
    action: TurnAllocation,
    reward: f64,
    next_state: ConversationState,
}

impl RLTurnAllocator {
    pub async fn allocate_turn(&self, state: ConversationState) -> Result<TurnAllocation> {
        // Get model prediction
        let allocation = self.model.predict(&state);
        
        // Apply exploration strategy
        let final_allocation = if self.should_explore() {
            self.explore_alternative(allocation)
        } else {
            allocation
        };
        
        // Store for learning
        self.store_decision(state, final_allocation.clone()).await;
        
        Ok(final_allocation)
    }
    
    pub async fn learn_from_feedback(&mut self, feedback: ConversationFeedback) {
        // Calculate rewards
        let rewards = self.calculate_rewards(feedback);
        
        // Update experiences with rewards
        self.experience_buffer.update_rewards(rewards).await;
        
        // Train model
        if self.should_train() {
            let batch = self.experience_buffer.sample(self.training_config.batch_size);
            self.model.update(batch);
        }
    }
    
    fn calculate_rewards(&self, feedback: ConversationFeedback) -> HashMap<ActionId, f64> {
        let mut rewards = HashMap::new();
        
        for (action_id, outcome) in feedback.outcomes {
            let reward = 
                outcome.participation_balance * self.training_config.balance_weight +
                outcome.conversation_quality * self.training_config.quality_weight +
                outcome.efficiency * self.training_config.efficiency_weight -
                outcome.conflicts * self.training_config.conflict_penalty;
            
            rewards.insert(action_id, reward);
        }
        
        rewards
    }
}
```

### Game-Theoretic Turn-Taking

Nash equilibrium-based allocation:

```rust
pub struct GameTheoreticAllocator {
    /// Game configuration
    game: TurnTakingGame,
    
    /// Equilibrium solver
    solver: Arc<dyn EquilibriumSolver>,
    
    /// Strategy profiles
    strategies: Arc<RwLock<HashMap<ParticipantId, Strategy>>>,
}

pub struct TurnTakingGame {
    /// Payoff matrix
    payoffs: PayoffMatrix,
    
    /// Game type
    game_type: GameType,
    
    /// Information structure
    information: InformationStructure,
}

pub enum GameType {
    /// Zero-sum game
    ZeroSum,
    
    /// Cooperative game
    Cooperative,
    
    /// Non-cooperative
    NonCooperative,
    
    /// Repeated game
    Repeated {
        rounds: usize,
        discount_factor: f64,
    },
}

pub struct Strategy {
    /// Pure or mixed strategy
    strategy_type: StrategyType,
    
    /// Action probabilities
    actions: HashMap<Action, f64>,
}

impl GameTheoreticAllocator {
    pub async fn compute_equilibrium(&self) -> Result<Equilibrium> {
        let current_strategies = self.strategies.read().await.clone();
        
        match self.game.game_type {
            GameType::ZeroSum => {
                self.solver.solve_zero_sum(&self.game.payoffs).await
            }
            
            GameType::Cooperative => {
                self.solver.solve_cooperative(
                    &self.game.payoffs,
                    &current_strategies
                ).await
            }
            
            GameType::Repeated { rounds, discount_factor } => {
                self.solver.solve_repeated(
                    &self.game.payoffs,
                    rounds,
                    discount_factor
                ).await
            }
            
            _ => self.solver.solve_nash(&self.game.payoffs, &current_strategies).await
        }
    }
    
    pub async fn allocate_based_on_equilibrium(&self) -> Result<TurnAllocation> {
        let equilibrium = self.compute_equilibrium().await?;
        
        // Sample from equilibrium strategies
        let mut allocation = TurnAllocation::new();
        
        for (participant, strategy) in equilibrium.strategies {
            if strategy.should_request_turn() {
                allocation.grant_turn(participant);
            }
        }
        
        Ok(allocation)
    }
}
```

## Performance Considerations

### Algorithm Complexity

Different algorithms have different complexity:

```rust
pub struct ComplexityAnalyzer {
    pub fn analyze_algorithm(algorithm: &TurnAlgorithm) -> ComplexityProfile {
        match algorithm {
            TurnAlgorithm::Sequential => ComplexityProfile {
                time: Complexity::O(1),  // Constant time operations
                space: Complexity::O(n),  // Queue storage
                communication: Complexity::O(1),
            },
            
            TurnAlgorithm::Auction => ComplexityProfile {
                time: Complexity::O(n_log_n),  // Sorting bids
                space: Complexity::O(n),
                communication: Complexity::O(n),  // Broadcast to all
            },
            
            TurnAlgorithm::Consensus => ComplexityProfile {
                time: Complexity::O(n_squared),  // All-to-all communication
                space: Complexity::O(n),
                communication: Complexity::O(n_squared),
            },
            
            TurnAlgorithm::GameTheoretic => ComplexityProfile {
                time: Complexity::Exponential(n),  // Nash equilibrium
                space: Complexity::O(n_squared),
                communication: Complexity::O(n),
            },
        }
    }
}
```

### Optimization Strategies

Making algorithms more efficient:

```rust
pub struct AlgorithmOptimizer {
    /// Caching for repeated computations
    cache: Arc<ComputationCache>,
    
    /// Approximation strategies
    approximations: ApproximationSettings,
    
    /// Parallelization configuration
    parallel_config: ParallelConfig,
}

impl AlgorithmOptimizer {
    pub async fn optimize_allocation(
        &self,
        algorithm: &mut dyn TurnAlgorithm,
        state: ConversationState,
    ) -> Result<TurnAllocation> {
        // Check cache first
        if let Some(cached) = self.cache.get(&state).await {
            return Ok(cached);
        }
        
        // Use approximation for large participant counts
        let result = if state.participant_count() > self.approximations.threshold {
            algorithm.approximate_allocation(state, self.approximations.epsilon).await?
        } else {
            algorithm.exact_allocation(state).await?
        };
        
        // Cache result
        self.cache.put(state, result.clone()).await;
        
        Ok(result)
    }
}
```

## Integration and Compatibility

### Protocol Adapters

Adapting algorithms to different protocols:

```rust
pub trait TurnAlgorithmAdapter {
    fn adapt_for_protocol(&self, protocol: Protocol) -> Box<dyn TurnAlgorithm>;
}

pub struct UniversalAdapter {
    pub fn adapt(
        algorithm: Box<dyn TurnAlgorithm>,
        protocol: Protocol,
    ) -> Box<dyn TurnAlgorithm> {
        match protocol {
            Protocol::WebSocket => Box::new(WebSocketAdapter::new(algorithm)),
            Protocol::A2A => Box::new(A2AAdapter::new(algorithm)),
            Protocol::MCP => Box::new(MCPAdapter::new(algorithm)),
            _ => algorithm,
        }
    }
}
```

## Best Practices

### Algorithm Selection

1. **Consider Scale**: Simple algorithms for small groups, sophisticated for large
2. **Latency Requirements**: Real-time needs favor simple algorithms
3. **Fairness Goals**: Consensus/voting for fairness, auctions for efficiency
4. **Participant Types**: Different algorithms for human vs agent participants

### Implementation Guidelines

1. **Timeout Handling**: Always implement timeouts for turn requests
2. **Fallback Mechanisms**: Have fallback when primary algorithm fails
3. **Metrics Collection**: Track fairness, efficiency, and participation
4. **Testing**: Test with various participant counts and behaviors

## Relationships

- **Parent Nodes:** [elements/distributed_conversation_coordination.md]
- **Child Nodes:** None
- **Related Nodes:**
  - [elements/websocket_hub_architecture.md] - implements - Transport layer
  - [elements/mixed_participant_patterns.md] - uses - For different participant types
  - [elements/interruptible_agent_loops.md] - integrates - Interruption handling

## Conclusion

Dynamic turn-taking algorithms provide the foundation for managing multi-party conversations in distributed agent systems. From simple sequential approaches to sophisticated game-theoretic models, these algorithms enable fair, efficient allocation of speaking rights while handling the complexities of mixed human-agent participation. The choice of algorithm depends on specific requirements around fairness, efficiency, scalability, and the nature of participants involved.