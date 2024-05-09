use autd3::prelude::*;

use crate::print_msg_and_wait_for_key;

pub async fn force_fan_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(ForceFan::new(|_| true)).await?;
    print_msg_and_wait_for_key("ファンが動いていること");

    autd.send(ForceFan::new(|_| false)).await?;
    print_msg_and_wait_for_key("ファンが止まっていること");

    Ok(())
}
