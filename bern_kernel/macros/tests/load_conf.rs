
mod tests {
    use bern_kernel_macros::load_conf;

    #[test]
    fn load() {
        load_conf!();

        assert_eq!(MUTEX_POOL_SIZE, 32);
    }
}