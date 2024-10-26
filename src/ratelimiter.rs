use chrono::{DateTime, TimeDelta, Utc};
use linked_hash_set::LinkedHashSet;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Add;
use std::time::SystemTime;

pub struct Ratelimiter {
    limits: HashMap<SiteAction, Limits>
}

impl Ratelimiter {
    pub fn new() -> Self {
        let mut limits = HashMap::new();
        for action in ACTIONS {
            limits.insert(action, Limits {
                blocked_times: vec![],
                searchable_users: Default::default(),
            });
        }
        Ratelimiter {
            limits
        }
    }

    pub fn check_limited(&mut self, action: SiteAction, ip: &UniqueIdentifier) -> bool {
        let limit = self.limits.get_mut(&action).unwrap();
        let output = limit.check_limited(ip);
        limit.add_limit(ip, action.get_limit());
        output
    }

    pub fn clear(&mut self) {
        self.limits.clear();
        for action in ACTIONS {
            self.limits.insert(action, Limits {
                blocked_times: vec![],
                searchable_users: Default::default(),
            });
        }
    }
}

pub struct Limits {
    blocked_times: Vec<DateTime<Utc>>,
    searchable_users: LinkedHashSet<UniqueIdentifier>
}

#[derive(Hash, Ord, PartialOrd, Eq, PartialEq, Clone, Copy)]
pub enum UniqueIdentifier {
    Ip(SocketAddr),
    Discord(u64)
}

impl Limits {
    pub fn check_limited(&mut self, ip: &UniqueIdentifier) -> bool {
        while !self.blocked_times.is_empty() {
            if self.blocked_times[0] < DateTime::<Utc>::from(SystemTime::now()) {
                self.searchable_users.pop_front();
            }
        }

        self.searchable_users.contains(ip)
    }

    pub fn add_limit(&mut self, ip: &UniqueIdentifier, time: i64) {
        self.blocked_times.insert(self.blocked_times.len(), DateTime::from(SystemTime::now()).add(TimeDelta::new(time,0).unwrap()));
        self.searchable_users.insert(ip.clone());
    }
}

pub const ACTIONS: [SiteAction; 3] = [SiteAction::Search, SiteAction::Upload, SiteAction::Update];

#[derive(Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SiteAction {
    Search,
    Update,
    Upload
}

impl SiteAction {
    pub fn get_limit(&self) -> i64 {
        match self {
            SiteAction::Search => 1,
            SiteAction::Update => 60,
            SiteAction::Upload => 60*60*24,
        }
    }
}