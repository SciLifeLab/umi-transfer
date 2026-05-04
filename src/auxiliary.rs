use std::{thread, time::Instant};

pub fn timedrun<F, R>(msg: &str, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let measure = func();
    println!("{msg} after {:.1} seconds", start.elapsed().as_secs_f32());
    measure
}

pub fn threads_available() -> usize {
    thread::available_parallelism()
        .map(|cores| cores.get())
        .unwrap_or_else(|_| {
            eprintln!(
                "Failed to determine number of available threads. Please specify manually with --threads."
            ); 1})
}

/// Compute thread allocations for the two parallel subsystems given a global `--threads` budget.
///
/// Returns `(paraseq_threads, gzp_threads_per_writer)`.
///
/// Fixed overhead that is subtracted before allocation:
///   - 1 main thread  (drives the streaming ordered write)
///   - 1 reader coordination thread  (spawned to call `process_parallel_*`)
///   - 1 gzp background thread per writer  (spawned at `create_writer` time, only when gzp uses
///     parallel mode, i.e. when the per-writer allocation is ≥ 2)
///
/// The remaining budget is split equally among `num_writers + 1` consumers (paraseq counts as
/// one unit).  When the per-writer share would be < 2, gzp falls back to its synchronous
/// single-threaded mode (no background thread), so paraseq receives the larger budget of
/// `num_threads − 2`.
pub fn compute_thread_budgets(num_threads: usize, num_writers: usize) -> (usize, usize) {
    // Subtract main + reader coord + gzp bg threads (one per writer)
    let remaining = num_threads.saturating_sub(2 + num_writers);
    let gzp_per = remaining / (num_writers + 1);

    if gzp_per >= 2 {
        // gzp uses parallel mode: 1 bg thread + gzp_per compression workers per writer.
        // Total = 2 + num_writers(bg) + paraseq + num_writers * gzp_per  ==  num_threads exactly.
        let paraseq = remaining.saturating_sub(gzp_per * num_writers).max(1);
        (paraseq, gzp_per)
    } else {
        // gzp falls back to SyncZ (compression is inline on the main thread, no bg thread).
        // The num_writers bg-thread slots we reserved are free, so paraseq gets num_threads − 2.
        let paraseq = num_threads.saturating_sub(2).max(1);
        (paraseq, 1)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_threads_available_returns_positive_number() {
        let threads = threads_available();
        assert!(threads > 0);
    }

    #[test]
    fn test_budgets_9_threads_3file() {
        // N=9, 2 writers: remaining=5, gzp_per=5/3=1 → SyncZ, paraseq=7
        let (paraseq, gzp) = compute_thread_budgets(9, 2);
        assert_eq!(gzp, 1);
        assert_eq!(paraseq, 7);
    }

    #[test]
    fn test_budgets_9_threads_2file() {
        // N=9, 1 writer: remaining=6, gzp_per=6/2=3 → parallel, paraseq=3
        let (paraseq, gzp) = compute_thread_budgets(9, 1);
        assert_eq!(gzp, 3);
        assert_eq!(paraseq, 3);
    }

    #[test]
    fn test_budgets_11_threads_3file() {
        // N=11, 2 writers: remaining=7, gzp_per=7/3=2 → parallel, paraseq=3
        let (paraseq, gzp) = compute_thread_budgets(11, 2);
        assert_eq!(gzp, 2);
        assert_eq!(paraseq, 3);
    }

    #[test]
    fn test_budgets_11_threads_2file() {
        // N=11, 1 writer: remaining=8, gzp_per=8/2=4 → parallel, paraseq=4
        let (paraseq, gzp) = compute_thread_budgets(11, 1);
        assert_eq!(gzp, 4);
        assert_eq!(paraseq, 4);
    }

    #[test]
    fn test_budgets_32_threads_3file() {
        // N=32, 2 writers: remaining=28, gzp_per=28/3=9 → parallel, paraseq=10
        let (paraseq, gzp) = compute_thread_budgets(32, 2);
        assert_eq!(gzp, 9);
        assert_eq!(paraseq, 10);
    }

    #[test]
    fn test_budgets_32_threads_2file() {
        // N=32, 1 writer: remaining=29, gzp_per=29/2=14 → parallel, paraseq=15
        let (paraseq, gzp) = compute_thread_budgets(32, 1);
        assert_eq!(gzp, 14);
        assert_eq!(paraseq, 15);
    }

    #[test]
    fn test_budgets_never_zero() {
        for n in 4..=64 {
            for w in 1..=2 {
                let (paraseq, gzp) = compute_thread_budgets(n, w);
                assert!(paraseq >= 1, "paraseq=0 for n={n}, w={w}");
                assert!(gzp >= 1, "gzp=0 for n={n}, w={w}");
            }
        }
    }

    #[test]
    fn test_budgets_parallel_mode_exact_total() {
        // When gzp is in parallel mode, verify the thread count adds up to exactly num_threads.
        for n in 4..=64 {
            for w in 1..=2_usize {
                let (paraseq, gzp) = compute_thread_budgets(n, w);
                if gzp >= 2 {
                    // 2 (main+reader) + w (gzp bg) + paraseq + w * gzp == n
                    let total = 2 + w + paraseq + w * gzp;
                    assert_eq!(total, n, "total={total} ≠ n={n} for w={w}");
                }
            }
        }
    }
}
