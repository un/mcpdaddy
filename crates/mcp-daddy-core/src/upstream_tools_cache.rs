use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct CachedUpstreamTools {
    pub upstream_id: String,
    pub tools: Vec<Value>,
    pub fetched_at: SystemTime,
}

#[derive(Debug, Default)]
struct ToolsCache {
    by_upstream: HashMap<String, CachedUpstreamTools>,
}

#[derive(Clone, Default)]
pub struct UpstreamToolsCacheStore {
    inner: Arc<RwLock<ToolsCache>>,
}

impl UpstreamToolsCacheStore {
    pub fn get(&self, upstream_id: &str) -> Option<CachedUpstreamTools> {
        let guard = self.inner.read().expect("tools cache rwlock poisoned");
        guard.by_upstream.get(upstream_id).cloned()
    }

    pub fn set(&self, upstream_id: impl Into<String>, tools: Vec<Value>) {
        let upstream_id = upstream_id.into();
        let mut guard = self.inner.write().expect("tools cache rwlock poisoned");
        guard.by_upstream.insert(
            upstream_id.clone(),
            CachedUpstreamTools {
                upstream_id,
                tools,
                fetched_at: SystemTime::now(),
            },
        );
    }

    pub fn clear(&self, upstream_id: &str) {
        let mut guard = self.inner.write().expect("tools cache rwlock poisoned");
        guard.by_upstream.remove(upstream_id);
    }
}
