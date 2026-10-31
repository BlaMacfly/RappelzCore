//! One CPU job and one result at most. No graphics or asset APIs enter the worker thread.
use super::audit::AuditHandle;
use serde_json::{json, Value};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    mpsc::{self, Receiver, SyncSender},
    Arc,
};
use std::thread::JoinHandle;
use std::time::Instant;

pub(super) type Key = (i32, i32);
#[derive(Clone)]
struct Token {
    cancel: Arc<AtomicBool>,
    started: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
    start_us: Arc<AtomicU64>,
    progress: Option<Arc<AtomicUsize>>,
}
impl Token {
    fn running(&self) -> bool {
        self.started.load(Ordering::Acquire) && !self.finished.load(Ordering::Acquire)
    }
    fn rows(&self) -> usize {
        self.progress
            .as_ref()
            .map_or(0, |p| p.load(Ordering::Relaxed))
    }
}
struct Request<I> {
    id: u64,
    key: Key,
    input: I,
    token: Token,
}
pub(super) struct Finished<O> {
    pub id: u64,
    pub key: Key,
    pub value: Option<O>,
    pub panicked: bool,
    pub elapsed_us: u64,
}
pub(super) struct Worker<I: Send + 'static, O: Send + 'static> {
    tx: Option<SyncSender<Request<I>>>,
    rx: Option<Receiver<Finished<O>>>,
    thread: Option<JoinHandle<()>>,
    active: Option<(u64, Key, Token)>,
    audit: AuditHandle,
    next: u64,
}

impl<I: Send + 'static, O: Send + 'static> Worker<I, O> {
    pub fn new(work: fn(I, &AtomicBool) -> Option<O>) -> std::io::Result<Self> {
        let (tx, input) = mpsc::sync_channel::<Request<I>>(1);
        let (output, rx) = mpsc::sync_channel(1);
        let audit = AuditHandle::new();
        let log = audit.clone();
        let thread = std::thread::Builder::new()
            .name("terrain-blend".into())
            .spawn(move || {
                while let Ok(job) = input.recv() {
                    let start = Instant::now();
                    let value = if job.token.cancel.load(Ordering::Relaxed) {
                        Ok(None)
                    } else {
                        log.record(
                            "started",
                            job.id,
                            job.key,
                            job.token.rows(),
                            true,
                            false,
                            false,
                        );
                        job.token
                            .start_us
                            .store(log.elapsed_us(), Ordering::Relaxed);
                        job.token.started.store(true, Ordering::Release);
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            work(job.input, &job.token.cancel)
                        }))
                    };
                    let panicked = value.is_err();
                    let produced = value.as_ref().is_ok_and(|v| v.is_some());
                    let value = if job.token.cancel.load(Ordering::Relaxed) {
                        None
                    } else {
                        value.ok().flatten()
                    };
                    log.record(
                        if panicked { "failed" } else { "finished" },
                        job.id,
                        job.key,
                        job.token.rows(),
                        false,
                        produced,
                        job.token.cancel.load(Ordering::Relaxed),
                    );
                    job.token.finished.store(true, Ordering::Release);
                    if output
                        .send(Finished {
                            id: job.id,
                            key: job.key,
                            value,
                            panicked,
                            elapsed_us: start.elapsed().as_micros().min(u64::MAX as u128) as u64,
                        })
                        .is_err()
                    {
                        break;
                    }
                }
            })?;
        Ok(Self {
            tx: Some(tx),
            rx: Some(rx),
            thread: Some(thread),
            active: None,
            next: 0,
            audit,
        })
    }
    pub fn idle(&self) -> bool {
        self.active.is_none()
    }
    pub fn audit(&self) -> AuditHandle {
        self.audit.clone()
    }
    pub fn active_snapshot(&self) -> Value {
        self.active.as_ref().map(|(id,key,t)|json!({"id":id,"key":[key.0,key.1],"running":t.running(),
            "started":t.started.load(Ordering::Acquire),"finished":t.finished.load(Ordering::Acquire),
            "rows":t.rows(),"cancelled":t.cancel.load(Ordering::Relaxed)})).unwrap_or(Value::Null)
    }
    pub fn submit(&mut self, key: Key, input: I) -> bool {
        self.submit_tracked(key, input, None)
    }
    pub fn submit_tracked(
        &mut self,
        key: Key,
        input: I,
        progress: Option<Arc<AtomicUsize>>,
    ) -> bool {
        if self.active.is_some() {
            return false;
        }
        let token = Token {
            cancel: Arc::new(AtomicBool::new(false)),
            started: Arc::new(AtomicBool::new(false)),
            finished: Arc::new(AtomicBool::new(false)),
            start_us: Arc::new(AtomicU64::new(0)),
            progress,
        };
        let id = self.next.wrapping_add(1);
        self.audit
            .record("submitted", id, key, 0, false, false, false);
        if self
            .tx
            .as_ref()
            .unwrap()
            .try_send(Request {
                id,
                key,
                input,
                token: token.clone(),
            })
            .is_err()
        {
            self.audit
                .record("submit_failed", id, key, 0, false, false, false);
            return false;
        }
        self.next = id;
        self.active = Some((id, key, token));
        true
    }
    pub fn cancel_unless(&self, keep: impl Fn(Key) -> bool) {
        self.cancel_with("cancel_view", keep);
    }
    fn cancel_with(&self, event: &'static str, keep: impl Fn(Key) -> bool) {
        if let Some((id, key, t)) = &self.active {
            if !keep(*key) && !t.cancel.load(Ordering::Relaxed) {
                self.audit
                    .record(event, *id, *key, t.rows(), t.running(), false, true);
                t.cancel.store(true, Ordering::Relaxed);
            }
        }
    }
    pub fn poll(&mut self) -> Option<Finished<O>> {
        let mut result = self.rx.as_ref()?.try_recv().ok()?;
        let Some((id, key, token)) = self.active.take() else {
            return None;
        };
        if id != result.id || key != result.key || token.cancel.load(Ordering::Relaxed) {
            result.value = None;
        }
        self.audit.record(
            if result.value.is_some() {
                "polled_value"
            } else {
                "polled_cancelled"
            },
            result.id,
            result.key,
            token.rows(),
            false,
            result.value.is_some(),
            token.cancel.load(Ordering::Relaxed),
        );
        Some(result)
    }
}

impl<I: Send + 'static, O: Send + 'static> Drop for Worker<I, O> {
    fn drop(&mut self) {
        let (id, key, rows, running) = self
            .active
            .as_ref()
            .map(|(i, k, t)| (*i, *k, t.rows(), t.running()))
            .unwrap_or((0, (0, 0), 0, false));
        self.audit
            .record("shutdown_begin", id, key, rows, running, false, false);
        self.cancel_with("cancel_shutdown", |_| false);
        self.tx.take();
        self.rx.take(); // Unblocks a worker publishing a result before joining.
        if let Some(thread) = self.thread.take() {
            let event = if thread.join().is_ok() {
                "joined"
            } else {
                "join_failed"
            };
            self.audit.record(event, id, key, rows, false, false, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn wait<I: Send + 'static, O: Send + 'static>(w: &mut Worker<I, O>) -> Finished<O> {
        let end = Instant::now() + std::time::Duration::from_secs(2);
        loop {
            if let Some(v) = w.poll() {
                return v;
            }
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
    }
    #[test]
    fn result_identity_and_single_outstanding_job_are_preserved() {
        let mut w = Worker::new(|n: u32, _| Some(vec![n as u8; 16])).unwrap();
        assert!(w.submit((3, 4), 17));
        assert!(!w.submit((4, 5), 18));
        let r = wait(&mut w);
        assert_eq!(r.key, (3, 4));
        assert_eq!(r.value.unwrap(), vec![17; 16]);
        assert!(w.idle());
        assert!(w.submit((4, 5), 18));
        let next = wait(&mut w);
        assert_ne!(r.id, next.id);
        assert_eq!(next.key, (4, 5));
    }
    #[test]
    fn cancellation_never_publishes_a_stale_image() {
        let mut w = Worker::new(|_: (), cancel| {
            while !cancel.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            Some(vec![7; 16])
        })
        .unwrap();
        assert!(w.submit((1, 2), ()));
        w.cancel_unless(|_| false);
        let result = wait(&mut w);
        assert!(result.value.is_none());
        assert!(w.idle());
    }
    #[test]
    fn drop_cancels_and_joins_without_needing_to_poll() {
        let mut w = Worker::new(|_: (), cancel| {
            while !cancel.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            Some(())
        })
        .unwrap();
        w.submit((0, 0), ());
        drop(w);
        let mut done = Worker::new(|_: (), _| Some(vec![0; 16])).unwrap();
        done.submit((1, 1), ());
        // Even if its result slot fills, dropping the receiver lets shutdown finish.
        drop(done);
    }
    #[test]
    fn worker_failure_is_reported_and_does_not_poison_the_channel() {
        let mut w = Worker::new(|n: u32, _| {
            assert_ne!(n, 0, "controlled worker failure");
            Some(n)
        })
        .unwrap();
        w.submit((0, 0), 0);
        let failed = wait(&mut w);
        assert!(failed.panicked);
        assert!(failed.value.is_none());
        assert!(w.submit((1, 1), 5));
        assert_eq!(wait(&mut w).value, Some(5));
    }

    #[test]
    fn completed_unpolled_result_is_suppressed_after_cancellation() {
        let mut w = Worker::new(|_: (), _| Some(vec![9; 16])).unwrap();
        w.submit((2, 3), ());
        let end = Instant::now() + std::time::Duration::from_secs(2);
        while w.active_snapshot()["finished"] != true {
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
        w.cancel_unless(|_| false);
        assert!(wait(&mut w).value.is_none());
        let events = w.audit().snapshot();
        assert!(events["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event"] == "polled_cancelled"));
    }

    #[test]
    fn running_shutdown_is_joined_and_audit_survives_world_lifetime() {
        let rows = Arc::new(AtomicUsize::new(0));
        let mut w = Worker::new(|progress: Arc<AtomicUsize>, cancel| {
            progress.store(17, Ordering::Relaxed);
            while !cancel.load(Ordering::Relaxed) {
                std::thread::yield_now();
            }
            None::<Vec<u8>>
        })
        .unwrap();
        let audit = w.audit();
        w.submit_tracked((4, 5), rows.clone(), Some(rows.clone()));
        let end = Instant::now() + std::time::Duration::from_secs(2);
        while rows.load(Ordering::Relaxed) == 0 {
            assert!(Instant::now() < end);
            std::thread::yield_now();
        }
        drop(w);
        let snapshot = audit.snapshot();
        let events = snapshot["events"].as_array().unwrap();
        assert_eq!(events.last().unwrap()["event"], "joined");
        assert!(events
            .iter()
            .any(|e| e["event"] == "shutdown_begin" && e["running"] == true && e["rows"] == 17));
        assert!(events
            .iter()
            .any(|e| e["event"] == "finished" && e["cancelled"] == true && e["produced"] == false));
    }
}
