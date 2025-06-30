use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, prelude::*};

pub fn phase_corr_test<L: Link>(autd: &mut Controller<L, firmware::V12_1>) -> anyhow::Result<()> {
    let wavenumber = autd.environment.wavenumber();
    autd.send(PhaseCorrection::new(move |dev| {
        let p = dev.center() + Vector3::new(0.0, 0.0, 150.0 * mm);
        move |tr| Phase::from(-((p - tr.position()).norm() * wavenumber) * rad)
    }))?;
    autd.send((
        Sine::new(150. * Hz, Default::default()),
        Uniform {
            intensity: Intensity(0xFF),
            phase: Phase::ZERO,
        },
    ))?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");

    autd.send(PhaseCorrection::new(|_dev| |_tr| Phase(0)))?;
    autd.send(Static::default())?;
    Ok(())
}
