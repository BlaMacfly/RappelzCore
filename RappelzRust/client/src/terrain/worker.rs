//! One CPU job and one result at most. No graphics or asset APIs enter the worker thread.
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc::{self, Receiver, SyncSender},
    Arc,
};
use std::thread::JoinHandle;
use std::time::Instant;

pub(super) type Key = (i32, i32);
struct Request<I> {
    id: u64,
    key: Key,
    input: I,
    cancel: Arc<AtomicBool>,
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
    active: Option<(u64, Key, Arc<AtomicBool>)>,
    next: u64,
}

impl<I: Send + 'static, O: Send + 'static> Worker<I, O> {
    pub fn new(work: fn(I, &AtomicBool) -> Option<O>) -> std::io::Result<Self> {
        let (tx, input) = mpsc::sync_channel::<Request<I>>(1);
        let (output, rx) = mpsc::sync_channel(1);
        let thread = std::thread::Builder::new()
            .name("terrain-blend".into())
            .spawn(move || {
                while let Ok(job) = input.recv() {
                    let start = Instant::now();
                    let value = if job.cancel.load(Ordering::Relaxed) {
                        Ok(None)
                    } else {
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            work(job.input, &job.cancel)
                        }))
                    };
                    let panicked = value.is_err();
                    let value = if job.cancel.load(Ordering::Relaxed) {
                        None
                    } else {
                        value.ok().flatten()
                    };
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
        })
    }
    pub fn idle(&self) -> bool {
        self.active.is_none()
    }
    pub fn submit(&mut self, key: Key, input: I) -> bool {
        if self.active.is_some() {
            return false;
        }
        let cancel = Arc::new(AtomicBool::new(false));
        let id = self.next.wrapping_add(1);
        let request = Request {
            id,
            key,
            input,
            cancel: cancel.clone(),
        };
        if self.tx.as_ref().unwrap().try_send(request).is_err() {
            return false;
        }
        self.next = id;
        self.active = Some((id, key, cancel));
        true
    }
    pub fn cancel_unless(&self, keep: impl Fn(Key) -> bool) {
        if let Some((_, key, cancel)) = &self.active {
            if !keep(*key) {
                cancel.store(true, Ordering::Relaxed);
            }
        }
    }
    pub fn poll(&mut self) -> Option<Finished<O>> {
        let mut result = self.rx.as_ref()?.try_recv().ok()?;
        let Some((id, key, cancel)) = self.active.take() else {
            return None;
        };
        if id != result.id || key != result.key || cancel.load(Ordering::Relaxed) {
            result.value = None;
        }
        Some(result)
    }
}

impl<I: Send + 'static, O: Send + 'static> Drop for Worker<I, O> {
    fn drop(&mut self) {
        self.cancel_unless(|_| false);
        self.tx.take();
        self.rx.take(); // Unblocks a worker publishing a result before joining.
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
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
}
