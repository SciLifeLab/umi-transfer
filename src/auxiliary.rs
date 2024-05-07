use std::{thread,time::Instant};

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

pub fn threads_per_task(available_threads: usize, num_tasks: usize) -> usize {
    if available_threads <= 1 || available_threads <= num_tasks {
        1
    } else {
        // Subtract 1 for the main thread
        let threads_for_tasks = available_threads - 1; 
        // The result is already always rounded down towards zero for integer divisions using the / operator.
        let threads_per_task = threads_for_tasks / num_tasks;
        threads_per_task.max(1)
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
    fn test_threads_per_task_never_returns_less_than_one() {
        let threads_per_task = threads_per_task(1,3);
        assert!(threads_per_task == 1);
    }

    #[test]
    fn test_threads_per_task_splits_even_threads_correctly() {
        let threads_per_task = threads_per_task(8,3);
        assert!(threads_per_task == 2);
    }

    #[test]
    fn test_threads_per_task_splits_odd_threads_correctly() {
        let threads_per_task = threads_per_task(10,3);
        assert!(threads_per_task == 3);
    }
}
