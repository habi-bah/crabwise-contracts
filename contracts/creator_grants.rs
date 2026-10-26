// Creator Grants System - Issue #585
// Manages grant allocations for creators on crabwise

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Grant {
    pub id: u64,
    pub creator_id: String,
    pub amount: u64,
    pub description: String,
    pub approved: bool,
}

#[derive(Debug, Default)]
pub struct CreatorGrantsSystem {
    pub grants: HashMap<u64, Grant>,
    pub next_id: u64,
}

impl CreatorGrantsSystem {
    pub fn new() -> Self {
        CreatorGrantsSystem {
            grants: HashMap::new(),
            next_id: 1,
        }
    }

    pub fn submit_grant(&mut self, creator_id: String, amount: u64, description: String) -> u64 {
        let id = self.next_id;
        let grant = Grant {
            id,
            creator_id,
            amount,
            description,
            approved: false,
        };
        self.grants.insert(id, grant);
        self.next_id += 1;
        id
    }

    pub fn approve_grant(&mut self, grant_id: u64) -> bool {
        if let Some(grant) = self.grants.get_mut(&grant_id) {
            grant.approved = true;
            return true;
        }
        false
    }

    pub fn get_grant(&self, grant_id: u64) -> Option<&Grant> {
        self.grants.get(&grant_id)
    }
}