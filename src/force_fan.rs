use autd3::{core::link::Link, prelude::*};

use crate::print_msg_and_wait_for_key;

pub fn force_fan_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(ForceFan::new(|_| true))?;
    print_msg_and_wait_for_key("ファンが動いていること");

    autd.send(ForceFan::new(|_| false))?;
    print_msg_and_wait_for_key("ファンが止まっていること");

    Ok(())
}
