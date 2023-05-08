use std::time::Instant;

pub fn timedrun<F, R>(msg: &str, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let measure = func();
    println!("{msg} after {:.1} seconds", start.elapsed().as_secs_f32());
    measure
}
