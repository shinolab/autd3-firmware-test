use std::time::Duration;

use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, prelude::*};

pub fn debug_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(GPIOOutputs::new(|_dev, gpio| match gpio {
        GPIOOut::O0 => GPIOOutputType::BaseSignal,
        _ => GPIOOutputType::None,
    }))?;

    autd.send((
        Static::default(),
        autd3::gain::Custom::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (0, 0) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0xFF),
                },
                (0, 248) => Drive {
                    phase: Phase(0x80),
                    intensity: EmitIntensity(0x80),
                },
                (_, 0) => Drive {
                    phase: Phase(0x80),
                    intensity: EmitIntensity(0xFF),
                },
                (_, 248) => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0x80),
                },
                _ => Drive {
                    phase: Phase(0),
                    intensity: EmitIntensity(0),
                },
            }
        }),
    ))?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンに出力がないこと");

    autd.send(GPIOOutputs::new(|dev, gpio| match gpio {
        GPIOOut::O0 => GPIOOutputType::BaseSignal,
        GPIOOut::O1 => GPIOOutputType::PwmOut(&dev[0]),
        _ => GPIOOutputType::None,
    }))?;
    print_msg_and_wait_for_key(
        "各デバイスのGPIO[1]ピンにDuty比50%の矩形波が出力されており, 位相が半周期ずれていること",
    );

    autd.send(GPIOOutputs::new(|dev, gpio| match gpio {
        GPIOOut::O0 => GPIOOutputType::BaseSignal,
        GPIOOut::O1 => GPIOOutputType::PwmOut(&dev[248]),
        _ => GPIOOutputType::None,
    }))?;
    print_msg_and_wait_for_key(
        "各デバイスのGPIO[1]ピンにDuty比約17%の矩形波が出力されており, 位相が半周期ずれていること",
    );

    autd.send(GPIOOutputs::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (0, GPIOOut::O1) => GPIOOutputType::PwmOut(&dev[0]),
        (_, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (_, GPIOOut::O1) => GPIOOutputType::PwmOut(&dev[248]),
        _ => GPIOOutputType::None,
    }))?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンの出力矩形波の位相が揃っていること");

    autd.send(GPIOOutputs::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (0, GPIOOut::O1) => GPIOOutputType::PwmOut(&dev[1]),
        (_, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (_, GPIOOut::O1) => GPIOOutputType::PwmOut(&dev[2]),
        _ => GPIOOutputType::None,
    }))?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンに出力がないこと");

    print_msg_and_wait_for_key(
        "0番目のデバイスのGPIO[1]にSingleトリガをセットする.\n次に, Enterを押し, 2秒後にトリガがかかること.\nまた, 0番目のデバイスのGPIO[1]出力と1番目のデバイスのGPIO[1]出力が25usずれていること",
    );
    let trig_time = DcSysTime::now() + Duration::from_secs(2);
    autd.send(GPIOOutputs::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (0, GPIOOut::O1) => GPIOOutputType::SysTimeEq(trig_time),
        (_, GPIOOut::O0) => GPIOOutputType::BaseSignal,
        (_, GPIOOut::O1) => {
            GPIOOutputType::SysTimeEq(trig_time + autd3::core::defined::ULTRASOUND_PERIOD)
        }
        _ => GPIOOutputType::None,
    }))?;

    println!("Enterを押して進む...");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
