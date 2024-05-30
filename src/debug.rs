use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, driver::link::Link, prelude::*};

pub async fn debug_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(DebugSettings::new(|_dev, gpio| match gpio {
        GPIOOut::O0 => DebugType::BaseSignal,
        _ => DebugType::None,
    }))
    .await?;

    autd.send((
        Static::new(),
        autd3::gain::Custom::new(|dev| {
            let dev_idx = dev.idx();
            move |tr| match (dev_idx, tr.idx()) {
                (0, 0) => Drive::new(Phase::new(0), EmitIntensity::new(0xFF)),
                (0, 248) => Drive::new(Phase::new(0x80), EmitIntensity::new(0x80)),
                (1, 0) => Drive::new(Phase::new(0x80), EmitIntensity::new(0xFF)),
                (1, 248) => Drive::new(Phase::new(0), EmitIntensity::new(0x80)),
                _ => Drive::null(),
            }
        }),
    ))
    .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンに出力がないこと");

    autd.send(DebugSettings::new(|dev, gpio| match gpio {
        GPIOOut::O0 => DebugType::BaseSignal,
        GPIOOut::O1 => DebugType::PwmOut(&dev[0]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key(
        "各デバイスのGPIO[1]ピンにDuty比50%の矩形波が出力されており, 位相が半周期ずれていること",
    );

    autd.send(DebugSettings::new(|dev, gpio| match gpio {
        GPIOOut::O0 => DebugType::BaseSignal,
        GPIOOut::O1 => DebugType::PwmOut(&dev[248]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key(
        "各デバイスのGPIO[1]ピンにDuty比約17%の矩形波が出力されており, 位相が半周期ずれていること",
    );

    autd.send(DebugSettings::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => DebugType::BaseSignal,
        (0, GPIOOut::O1) => DebugType::PwmOut(&dev[0]),
        (1, GPIOOut::O0) => DebugType::BaseSignal,
        (1, GPIOOut::O1) => DebugType::PwmOut(&dev[248]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンの出力矩形波の位相が揃っていること");

    autd.send(DebugSettings::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => DebugType::BaseSignal,
        (0, GPIOOut::O1) => DebugType::PwmOut(&dev[1]),
        (1, GPIOOut::O0) => DebugType::BaseSignal,
        (1, GPIOOut::O1) => DebugType::PwmOut(&dev[2]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンに出力がないこと");

    Ok(())
}
