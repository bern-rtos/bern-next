pub mod main;
pub mod st_nucleo_f446;

pub fn stupid_wait(iterations: usize) {
    let mut i = 0;
    while i < iterations {
        i += 1;
    }
}