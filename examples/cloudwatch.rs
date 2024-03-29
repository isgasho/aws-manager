use std::{thread, time};

use aws_manager::cloudwatch;
use log::info;

/// cargo run --example cloudwatch
fn main() {
    // ref. https://github.com/env-logger-rs/env_logger/issues/47
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    );

    macro_rules! ab {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    info!("creating AWS CloudWatch resources!");

    let shared_config = ab!(aws_manager::load_config(None)).unwrap();
    let cw_manager = cloudwatch::Manager::new(&shared_config);
    let log_group_name = random_manager::string(15);

    // error should be ignored if it does not exist
    let ret = ab!(cw_manager.delete_log_group("invalid_id"));
    assert!(ret.is_ok());

    ab!(cw_manager.create_log_group(&log_group_name)).unwrap();

    thread::sleep(time::Duration::from_secs(5));

    ab!(cw_manager.delete_log_group(&log_group_name)).unwrap();

    thread::sleep(time::Duration::from_secs(5));

    // error should be ignored if it's already scheduled for delete
    ab!(cw_manager.delete_log_group(&log_group_name)).unwrap();
}
