use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpstreamStatus {
    Unknown,
    Healthy,
    Unhealthy,
}

#[derive(Debug, Clone)]
pub struct UpstreamRuntime {
    pub upstream_id: String,
    pub status: UpstreamStatus,
    pub last_changed_at: SystemTime,
    pub last_tool_call_at: Option<SystemTime>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClientSessionRuntime {
    pub session_id: String,
    pub profile_id: String,
    pub connected_at: SystemTime,
    pub last_call_at: Option<SystemTime>,
}

#[derive(Debug, Clone)]
pub struct RuntimeSnapshot {
    pub upstreams: Vec<UpstreamRuntime>,
    pub client_sessions: Vec<ClientSessionRuntime>,
    pub last_call_by_profile: HashMap<String, SystemTime>,
}

#[derive(Debug, Default)]
struct RuntimeState {
    upstreams: HashMap<String, UpstreamRuntime>,
    client_sessions: HashMap<String, ClientSessionRuntime>,
    last_call_by_profile: HashMap<String, SystemTime>,
}

#[derive(Clone, Default)]
pub struct RuntimeStateStore {
    inner: Arc<RwLock<RuntimeState>>,
}

impl RuntimeStateStore {
    pub fn snapshot(&self) -> RuntimeSnapshot {
        let state = self.inner.read().expect("runtime state rwlock poisoned");

        RuntimeSnapshot {
            upstreams: state.upstreams.values().cloned().collect(),
            client_sessions: state.client_sessions.values().cloned().collect(),
            last_call_by_profile: state.last_call_by_profile.clone(),
        }
    }

    pub fn set_upstream_status(
        &self,
        upstream_id: impl Into<String>,
        status: UpstreamStatus,
        last_error: Option<String>,
    ) {
        let upstream_id = upstream_id.into();
        let mut state = self.inner.write().expect("runtime state rwlock poisoned");

        let now = SystemTime::now();
        let entry = state
            .upstreams
            .entry(upstream_id.clone())
            .or_insert_with(|| UpstreamRuntime {
                upstream_id,
                status: UpstreamStatus::Unknown,
                last_changed_at: now,
                last_tool_call_at: None,
                last_error: None,
            });

        if entry.status != status {
            entry.status = status;
            entry.last_changed_at = now;
        }

        entry.last_error = last_error;
    }

    pub fn record_upstream_tool_call(&self, upstream_id: impl Into<String>) {
        let upstream_id = upstream_id.into();
        let mut state = self.inner.write().expect("runtime state rwlock poisoned");

        let now = SystemTime::now();
        let entry = state
            .upstreams
            .entry(upstream_id.clone())
            .or_insert_with(|| UpstreamRuntime {
                upstream_id,
                status: UpstreamStatus::Unknown,
                last_changed_at: now,
                last_tool_call_at: None,
                last_error: None,
            });

        entry.last_tool_call_at = Some(now);
    }

    pub fn start_client_session(
        &self,
        session_id: impl Into<String>,
        profile_id: impl Into<String>,
    ) {
        let session_id = session_id.into();
        let profile_id = profile_id.into();

        let mut state = self.inner.write().expect("runtime state rwlock poisoned");

        let now = SystemTime::now();
        state.client_sessions.insert(
            session_id.clone(),
            ClientSessionRuntime {
                session_id,
                profile_id,
                connected_at: now,
                last_call_at: None,
            },
        );
    }

    pub fn end_client_session(&self, session_id: &str) {
        let mut state = self.inner.write().expect("runtime state rwlock poisoned");
        state.client_sessions.remove(session_id);
    }

    pub fn record_downstream_call(&self, session_id: &str) {
        let mut state = self.inner.write().expect("runtime state rwlock poisoned");

        let now = SystemTime::now();
        let profile_id = match state.client_sessions.get_mut(session_id) {
            Some(sess) => {
                sess.last_call_at = Some(now);
                sess.profile_id.clone()
            }
            None => return,
        };

        state.last_call_by_profile.insert(profile_id, now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_is_non_persistent_and_mutable() {
        let store = RuntimeStateStore::default();
        store.set_upstream_status("github", UpstreamStatus::Healthy, None);
        store.start_client_session("s1", "default");
        store.record_downstream_call("s1");

        let snap = store.snapshot();
        assert_eq!(snap.upstreams.len(), 1);
        assert_eq!(snap.client_sessions.len(), 1);
        assert!(snap.last_call_by_profile.contains_key("default"));

        store.end_client_session("s1");
        let snap2 = store.snapshot();
        assert_eq!(snap2.client_sessions.len(), 0);
    }
}
