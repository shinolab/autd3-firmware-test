use autd3::{core::link::Link, prelude::*};

use crate::print_msg_and_wait_for_key;

pub fn pwe_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(GPIOOutputs::new(|dev, gpio| match gpio {
        GPIOOut::O0 => GPIOOutputType::PwmOut(&dev[0]),
        GPIOOut::O1 => GPIOOutputType::PwmOut(&dev[248]),
        _ => GPIOOutputType::None,
    }))?;

    autd.send(PulseWidthEncoder::new(|_| {
        |i| match i.0 {
            0 => PulseWidth::from_duty(6.25 / 100.).unwrap(),
            1 => PulseWidth::from_duty(12.5 / 100.).unwrap(),
            2 => PulseWidth::from_duty(18.75 / 100.).unwrap(),
            3 => PulseWidth::from_duty(25. / 100.).unwrap(),
            _ => PulseWidth::from_duty(0.5).unwrap(),
        }
    }))?;
    autd.send((
        Static::default(),
        autd3::gain::Custom::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (0, 0) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0),
                },
                (0, 248) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(1),
                },
                (_, 0) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(2),
                },
                (_, 248) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(3),
                },
                _ => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0),
                },
            }
        }),
    ))?;
    print_msg_and_wait_for_key(
        "0番目のデバイスのGPIO[0]出力, 0番目のデバイスのGPIO[1]出力, 1番目のデバイスのGPIO[0]出力, 1番目のデバイスのGPIO[1]出力矩形波のDuty比がそれぞれ6.25, 12.5%, 18.75%, 25%であること",
    );

    autd.send(PulseWidthEncoder::new(|_| |_| PulseWidth::new(0).unwrap()))?;
    autd.send((
        Static::default(),
        Uniform::new(EmitIntensity::MAX, Phase::ZERO),
    ))?;
    print_msg_and_wait_for_key("各デバイスのGPIO[0]とGPIO[1]ピンに出力がないこと");

    autd.send(PulseWidthEncoder::default())?;
    autd.send((
        Static::default(),
        autd3::gain::Custom::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (_, 0) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0),
                },
                (_, 248) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(255),
                },
                _ => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0),
                },
            }
        }),
    ))?;
    print_msg_and_wait_for_key(
        "各デバイスのGPIO[0]出力, GPIO[1]出力出力矩形波のDuty比がそれぞれ0%, 50%であること",
    );

    Ok(())
}
