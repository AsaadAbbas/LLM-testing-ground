use chronokv_core::{NodeId, NodeRole, Term};
use serde::{Deserialize, Serialize};

/// Election state for leader election protocol.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectionState {
    pub node_id: NodeId,
    pub current_term: Term,
    pub voted_for: Option<NodeId>,
    pub role: NodeRole,
    pub votes_received: u32,
    pub total_nodes: u32,
}

impl ElectionState {
    pub fn new(node_id: NodeId, total_nodes: u32) -> Self {
        Self {
            node_id,
            current_term: 0,
            voted_for: None,
            role: NodeRole::Follower,
            votes_received: 0,
            total_nodes,
        }
    }

    /// Start a new election.
    pub fn start_election(&mut self) {
        self.current_term += 1;
        self.role = NodeRole::Candidate;
        self.voted_for = Some(self.node_id.clone());
        self.votes_received = 1; // Vote for self
    }

    /// Receive a vote.
    pub fn receive_vote(&mut self) -> bool {
        self.votes_received += 1;
        let majority = self.total_nodes / 2 + 1;

        if self.votes_received >= majority {
            self.role = NodeRole::Leader;
            true
        } else {
            false
        }
    }

    /// Step down to follower.
    pub fn step_down(&mut self, new_term: Term) {
        self.current_term = new_term;
        self.role = NodeRole::Follower;
        self.voted_for = None;
        self.votes_received = 0;
    }

    pub fn is_leader(&self) -> bool {
        self.role == NodeRole::Leader
    }

    pub fn is_candidate(&self) -> bool {
        self.role == NodeRole::Candidate
    }
}

/// Vote request message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteRequest {
    pub term: Term,
    pub candidate_id: NodeId,
}

/// Vote response message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoteResponse {
    pub term: Term,
    pub voter_id: NodeId,
    pub granted: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_election_majority() {
        let mut state = ElectionState::new("node1".to_string(), 3);
        state.start_election();

        assert_eq!(state.votes_received, 1);
        assert!(state.is_candidate());

        // Need 2 votes for majority of 3
        let won = state.receive_vote();
        assert!(won);
        assert!(state.is_leader());
    }

    #[test]
    fn test_step_down() {
        let mut state = ElectionState::new("node1".to_string(), 3);
        state.start_election();
        state.receive_vote();
        assert!(state.is_leader());

        state.step_down(5);
        assert!(!state.is_leader());
        assert_eq!(state.current_term, 5);
    }
}
