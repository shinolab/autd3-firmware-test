use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn phase_filter_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(ConfigurePhaseFilter::additive(|dev, tr| {
        tr.align_phase_at(
            dev.center() + Vector3::new(0.0, 0.0, 150.0 * MILLIMETER),
            dev.sound_speed,
        )
    }))
    .await?;
    autd.send((
        Sine::new(150.),
        Uniform::new(0xFF).with_phase(Phase::new(0)),
    ))
    .await?;
    print_msg_and_wait_for_key(
    "Check that the focal points are generated 150mm directly above the center of each device by your hands."
);

    autd.send(ConfigurePhaseFilter::additive(|_dev, _tr| Phase::new(0)))
        .await?;
    autd.send(Static::new()).await?;
    Ok(())
}
