pub mod protocol;

use chronokv_core::{
    ChronoError, Entry, NodeId, NodeRole, ReplicationMessage, Term, Timestamp,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Manages replication state for a node.
///
/// Supports leader-follower replication with term-based consistency.
/// The leader accepts writes and replicates to followers.
/// Followers reject writes and serve reads (if configured).
pub struct ReplicationManager {
    pub node_id: NodeId,
    pub role: Arc<RwLock<NodeRole>>,
    pub current_term: Arc<RwLock<Term>>,
    pub leader_id: Arc<RwLock<Option<NodeId>>>,

    /// Committed entries with their original commit term.
    committed_entries: Arc<RwLock<Vec<(Term, Entry)>>>,

    /// Last known timestamp per follower (for catch-up tracking).
    follower_progress: Arc<RwLock<HashMap<NodeId, Timestamp>>>,
}

impl ReplicationManager {
    pub fn new(node_id: NodeId, role: NodeRole) -> Self {
        let is_leader = role == NodeRole::Leader;
        Self {
            node_id: node_id.clone(),
            role: Arc::new(RwLock::new(role)),
            current_term: Arc::new(RwLock::new(1)),
            leader_id: Arc::new(RwLock::new(if is_leader {
                Some(node_id)
            } else {
                None
            })),
            committed_entries: Arc::new(RwLock::new(Vec::new())),
            follower_progress: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record an entry as committed at the current term.
    pub async fn commit_entry(&self, entry: Entry) -> Result<(), ChronoError> {
        let term = *self.current_term.read().await;
        let mut entries = self.committed_entries.write().await;
        entries.push((term, entry));
        Ok(())
    }

    /// Generate a heartbeat message (leader only).
    pub async fn make_heartbeat(&self) -> Result<ReplicationMessage, ChronoError> {
        let role = self.role.read().await;
        if *role != NodeRole::Leader {
            return Err(ChronoError::ReplicationError(
                "only leader can send heartbeats".to_string(),
            ));
        }

        let term = *self.current_term.read().await;
        let entries = self.committed_entries.read().await;
        let last_ts = entries.last().map(|(_, e)| e.timestamp).unwrap_or(0.0);

        Ok(ReplicationMessage::Heartbeat {
            term,
            leader_id: self.node_id.clone(),
            last_entry_timestamp: last_ts,
        })
    }

    /// Handle a catch-up request from a follower.
    ///
    /// Returns entries the follower is missing, starting from their
    /// last known timestamp.
    pub async fn handle_catch_up(
        &self,
        request_term: Term,
        follower_id: &str,
        last_known_ts: Timestamp,
    ) -> Result<ReplicationMessage, ChronoError> {
        let role = self.role.read().await;
        if *role != NodeRole::Leader {
            return Err(ChronoError::ReplicationError(
                "only leader handles catch-up".to_string(),
            ));
        }

        let entries = self.committed_entries.read().await;

        // Find entries the follower is missing
        let missing: Vec<Entry> = entries
            .iter()
            .filter(|(_, e)| e.timestamp > last_known_ts)
            .map(|(_, e)| e.clone())
            .collect();

        // Update follower progress tracking
        if let Some(last) = missing.last() {
            let mut progress = self.follower_progress.write().await;
            progress.insert(follower_id.to_string(), last.timestamp);
        }

        // Send the catch-up response with the entries' original commit terms
        let committed = self.committed_entries.read().await;
        let catch_up_entries: Vec<Entry> = committed
            .iter()
            .filter(|(_, e)| e.timestamp > last_known_ts)
            .map(|(_, e)| e.clone())
            .collect();

        // Use the original commit term for the response, not the current term
        let response_term = entries
            .iter()
            .filter(|(_, e)| e.timestamp > last_known_ts)
            .map(|(t, _)| *t)
            .next()
            .unwrap_or(request_term);

        Ok(ReplicationMessage::CatchUpResponse {
            term: response_term,
            entries: catch_up_entries,
        })
    }

    /// Handle an incoming replication message as a follower.
    pub async fn handle_message(
        &self,
        message: ReplicationMessage,
    ) -> Result<Option<Vec<Entry>>, ChronoError> {
        match message {
            ReplicationMessage::Heartbeat {
                term,
                leader_id,
                last_entry_timestamp: _,
            } => {
                let current_term = *self.current_term.read().await;
                if term < current_term {
                    // Stale heartbeat from old leader, ignore
                    tracing::debug!(
                        "Ignoring stale heartbeat: term {} < current {}",
                        term,
                        current_term
                    );
                    return Ok(None);
                }

                // Accept the leader
                *self.current_term.write().await = term;
                *self.leader_id.write().await = Some(leader_id);
                *self.role.write().await = NodeRole::Follower;

                Ok(None)
            }

            ReplicationMessage::AppendEntries {
                term,
                leader_id: _,
                entries,
            } => {
                let current_term = *self.current_term.read().await;
                if term < current_term {
                    // Reject entries from stale term
                    tracing::warn!(
                        "Rejecting AppendEntries: term {} < current term {}",
                        term,
                        current_term
                    );
                    return Ok(None);
                }

                *self.current_term.write().await = term;

                // Store committed entries
                let mut committed = self.committed_entries.write().await;
                for entry in &entries {
                    committed.push((term, entry.clone()));
                }

                Ok(Some(entries))
            }

            ReplicationMessage::CatchUpResponse { term, entries } => {
                let current_term = *self.current_term.read().await;
                if term < current_term {
                    // Reject catch-up from stale term
                    tracing::warn!(
                        "Rejecting CatchUpResponse: term {} < current term {}",
                        term,
                        current_term
                    );
                    return Ok(None);
                }

                // Apply caught-up entries
                let mut committed = self.committed_entries.write().await;
                for entry in &entries {
                    committed.push((term, entry.clone()));
                }

                Ok(Some(entries))
            }

            _ => Ok(None),
        }
    }

    /// Check if this node is the leader.
    pub async fn is_leader(&self) -> bool {
        *self.role.read().await == NodeRole::Leader
    }

    /// Get the current term.
    pub async fn term(&self) -> Term {
        *self.current_term.read().await
    }

    /// Get committed entry count.
    pub async fn committed_count(&self) -> usize {
        self.committed_entries.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chronokv_core::{Entry, NodeRole};

    #[tokio::test]
    async fn test_leader_heartbeat() {
        let leader = ReplicationManager::new("node1".to_string(), NodeRole::Leader);

        let entry = Entry::put("key1".to_string(), b"v1".to_vec(), 100.0);
        leader.commit_entry(entry).await.unwrap();

        let heartbeat = leader.make_heartbeat().await.unwrap();
        match heartbeat {
            ReplicationMessage::Heartbeat {
                term,
                leader_id,
                last_entry_timestamp,
            } => {
                assert_eq!(term, 1);
                assert_eq!(leader_id, "node1");
                assert_eq!(last_entry_timestamp, 100.0);
            }
            _ => panic!("expected heartbeat"),
        }
    }

    #[tokio::test]
    async fn test_follower_accepts_heartbeat() {
        let follower = ReplicationManager::new("node2".to_string(), NodeRole::Follower);

        let msg = ReplicationMessage::Heartbeat {
            term: 1,
            leader_id: "node1".to_string(),
            last_entry_timestamp: 100.0,
        };

        let result = follower.handle_message(msg).await.unwrap();
        assert!(result.is_none()); // heartbeat returns no entries
        assert_eq!(*follower.leader_id.read().await, Some("node1".to_string()));
    }

    #[tokio::test]
    async fn test_follower_rejects_stale_term() {
        let follower = ReplicationManager::new("node2".to_string(), NodeRole::Follower);
        *follower.current_term.write().await = 5;

        let msg = ReplicationMessage::AppendEntries {
            term: 3, // stale
            leader_id: "node1".to_string(),
            entries: vec![Entry::put("k".to_string(), b"v".to_vec(), 100.0)],
        };

        let result = follower.handle_message(msg).await.unwrap();
        assert!(result.is_none()); // rejected
    }
}
