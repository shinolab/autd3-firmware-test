use autd3::{driver::link::Link, prelude::*};

use crate::print_msg_and_wait_for_key;

pub async fn pwe_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(DebugSettings::new(|dev, gpio| match gpio {
        GPIOOut::O0 => DebugType::PwmOut(&dev[0]),
        GPIOOut::O1 => DebugType::PwmOut(&dev[248]),
        _ => DebugType::None,
    }))
    .await?;

    autd.send(PulseWidthEncoder::new(|_| {
        |i| match i {
            0 => 256 / 8,
            1..=127 => 256 / 8 * 2,
            128..=255 => 256 / 8 * 3,
            256..=383 => 256 / 8 * 4,
            _ => 256,
        }
    }))
    .await?;
    autd.send((
        Static::with_intensity(0xFF),
        autd3::gain::Custom::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (0, 0) => Drive::new(Phase::new(0), EmitIntensity::new(0)),
                (0, 248) => Drive::new(Phase::new(0), EmitIntensity::new(1)),
                (_, 0) => Drive::new(Phase::new(0), EmitIntensity::new(2)),
                (_, 248) => Drive::new(Phase::new(0), EmitIntensity::new(3)),
                _ => Drive::null(),
            }
        }),
    ))
    .await?;
    print_msg_and_wait_for_key("0番目のデバイスのGPIO[0]出力, 0番目のデバイスのGPIO[1]出力, 1番目のデバイスのGPIO[0]出力, 1番目のデバイスのGPIO[1]出力矩形波のDuty比がそれぞれ6.25, 12.5%, 18.75%, 25%であること");

    autd.send(PulseWidthEncoder::new(|_| |_| 0)).await?;
    autd.send((Static::with_intensity(0xFF), Uniform::new(0xFF)))
        .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[0]とGPIO[1]ピンに出力がないこと");

    autd.send((Static::with_intensity(0), Uniform::new(0)))
        .await?;
    autd.send(PulseWidthEncoder::default()).await?;

    Ok(())
}
