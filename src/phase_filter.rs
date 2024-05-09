use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn phase_filter_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(PhaseFilter::additive(|dev| {
        let wavenumber = dev.wavenumber();
        let p = dev.center() + Vector3::new(0.0, 0.0, 150.0 * MILLIMETER);
        move |tr| Phase::from_rad((p - tr.position()).norm() * wavenumber)
    }))
    .await?;
    autd.send((
        Sine::new(150.),
        Uniform::new(0xFF).with_phase(Phase::new(0)),
    ))
    .await?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");

    autd.send(PhaseFilter::additive(|_dev| |_tr| Phase::new(0)))
        .await?;
    autd.send(Static::new()).await?;
    Ok(())
}
