//! Bounded diagnostic metadata; retains neither terrain pixels nor GPU objects.
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::Instant;

const LIMIT: usize = 1024;
static NEXT: AtomicU64 = AtomicU64::new(1);

#[derive(Clone)]
pub(crate) struct AuditHandle(Arc<Audit>);
struct Audit {
    instance: u64,
    started: Instant,
    state: Mutex<State>,
}
#[derive(Default)]
struct State {
    events: Vec<Event>,
    overflow: usize,
}
struct Event {
    event: &'static str,
    id: u64,
    key: (i32, i32),
    rows: usize,
    running: bool,
    produced: bool,
    cancelled: bool,
    us: u64,
}

impl AuditHandle {
    pub(crate) fn instance(&self) -> u64 {
        self.0.instance
    }
    pub(super) fn new() -> Self {
        Self(Arc::new(Audit {
            instance: NEXT.fetch_add(1, Ordering::Relaxed),
            started: Instant::now(),
            state: Mutex::new(State::default()),
        }))
    }
    pub(super) fn elapsed_us(&self) -> u64 {
        self.0.started.elapsed().as_micros().min(u64::MAX as u128) as u64
    }
    pub(super) fn record(
        &self,
        event: &'static str,
        id: u64,
        key: (i32, i32),
        rows: usize,
        running: bool,
        produced: bool,
        cancelled: bool,
    ) {
        let mut state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.events.len() < LIMIT {
            state.events.push(Event {
                event,
                id,
                key,
                rows,
                running,
                produced,
                cancelled,
                us: self.elapsed_us(),
            });
        } else {
            state.overflow += 1;
        }
    }
    pub(crate) fn snapshot(&self) -> Value {
        let state = self.0.state.lock().unwrap_or_else(|e| e.into_inner());
        json!({"instance":self.0.instance,"limit":LIMIT,"overflow":state.overflow,
            "events":state.events.iter().enumerate().map(|(i,e)|json!({"order":i,"event":e.event,"id":e.id,
                "key":[e.key.0,e.key.1],"rows":e.rows,"running":e.running,"produced":e.produced,
                "cancelled":e.cancelled,"us":e.us})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn metadata_is_bounded_and_instance_ids_are_distinct() {
        let a = AuditHandle::new();
        let b = AuditHandle::new();
        for i in 0..LIMIT + 5 {
            a.record("test", i as u64, (1, 2), 0, false, false, false);
        }
        let s = a.snapshot();
        assert_eq!(s["events"].as_array().unwrap().len(), LIMIT);
        assert_eq!(s["overflow"], 5);
        assert_ne!(s["instance"], b.snapshot()["instance"]);
    }
}
