//! Wait for real session saves; never treat an elapsed grace delay as completion.
use std::time::Duration;
use anyhow::{bail, ensure, Result};
use tokio::{task::JoinSet, time::{timeout_at, Instant}};

pub async fn drain(sessions: &mut JoinSet<Result<()>>, timeout: Duration, already_failed: usize) -> Result<()> {
    let deadline = Instant::now() + timeout;
    let mut failed = already_failed;
    while !sessions.is_empty() {
        match timeout_at(deadline, sessions.join_next()).await {
            Ok(Some(Ok(Ok(())))) => {},
            Ok(Some(Ok(Err(error)))) => { failed += 1; tracing::error!("Session non sauvegardée correctement à l'arrêt : {error:#}"); },
            Ok(Some(Err(error))) => { failed += 1; tracing::error!("Session interrompue à l'arrêt : {error}"); },
            Ok(None) => break,
            Err(_) => {
                let pending = sessions.len();
                sessions.abort_all();
                while sessions.join_next().await.is_some() {}
                bail!("Arrêt incomplet : délai de sauvegarde dépassé pour {pending} session(s)");
            }
        }
    }
    ensure!(failed == 0, "Arrêt incomplet : {failed} session(s) en échec");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::{AtomicBool, AtomicUsize, Ordering}};
    #[tokio::test]
    async fn waits_for_every_save_instead_of_the_first_completed_session() {
        let mut sessions = JoinSet::new(); let saved = Arc::new(AtomicUsize::new(0));
        let (release, ready) = tokio::sync::oneshot::channel();
        let first = saved.clone(); sessions.spawn(async move { first.fetch_add(1, Ordering::SeqCst); Ok(()) });
        let second = saved.clone(); sessions.spawn(async move { ready.await.unwrap(); second.fetch_add(1, Ordering::SeqCst); Ok(()) });
        let mut drain = Box::pin(drain(&mut sessions, Duration::from_secs(1), 0));
        assert!(tokio::time::timeout(Duration::from_millis(20), &mut drain).await.is_err());
        assert_eq!(saved.load(Ordering::SeqCst), 1);
        release.send(()).unwrap(); drain.await.unwrap(); assert_eq!(saved.load(Ordering::SeqCst), 2);
        assert!(sessions.is_empty());
    }
    #[tokio::test]
    async fn failed_save_does_not_abandon_another_pending_save_or_report_success() {
        let mut sessions = JoinSet::new(); let saved = Arc::new(AtomicBool::new(false)); let other = saved.clone();
        sessions.spawn(async { anyhow::bail!("simulated failed COMMIT") });
        sessions.spawn(async move { tokio::time::sleep(Duration::from_millis(20)).await; other.store(true, Ordering::SeqCst); Ok(()) });
        assert!(drain(&mut sessions, Duration::from_secs(1), 0).await.is_err());
        assert!(saved.load(Ordering::SeqCst)); assert!(sessions.is_empty());
    }
    #[tokio::test]
    async fn timeout_is_an_error_and_all_remaining_session_futures_are_joined() {
        struct DropFlag(Arc<AtomicBool>); impl Drop for DropFlag { fn drop(&mut self) { self.0.store(true, Ordering::SeqCst); } }
        let mut sessions = JoinSet::new(); let dropped = Arc::new(AtomicBool::new(false)); let guard = DropFlag(dropped.clone());
        sessions.spawn(async move { let _guard = guard; std::future::pending::<()>().await; Ok(()) });
        assert!(drain(&mut sessions, Duration::from_millis(20), 0).await.is_err());
        assert!(sessions.is_empty()); assert!(dropped.load(Ordering::SeqCst));
    }
    #[tokio::test]
    async fn failed_session_already_reaped_during_the_shutdown_signal_is_not_forgotten() {
        let mut sessions = JoinSet::new();
        assert!(drain(&mut sessions, Duration::from_secs(1), 1).await.is_err());
    }
}
