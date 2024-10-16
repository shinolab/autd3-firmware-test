use std::time::Duration;

use crate::print_msg_and_wait_for_key;

use autd3::{driver::link::Link, prelude::*};

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
                (0, 0) => (Phase::new(0), EmitIntensity::new(0xFF)),
                (0, 248) => (Phase::new(0x80), EmitIntensity::new(0x80)),
                (_, 0) => (Phase::new(0x80), EmitIntensity::new(0xFF)),
                (_, 248) => (Phase::new(0), EmitIntensity::new(0x80)),
                _ => (Phase::new(0), EmitIntensity::new(0)),
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
        (_, GPIOOut::O0) => DebugType::BaseSignal,
        (_, GPIOOut::O1) => DebugType::PwmOut(&dev[248]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンの出力矩形波の位相が揃っていること");

    autd.send(DebugSettings::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => DebugType::BaseSignal,
        (0, GPIOOut::O1) => DebugType::PwmOut(&dev[1]),
        (_, GPIOOut::O0) => DebugType::BaseSignal,
        (_, GPIOOut::O1) => DebugType::PwmOut(&dev[2]),
        _ => DebugType::None,
    }))
    .await?;
    print_msg_and_wait_for_key("各デバイスのGPIO[1]ピンに出力がないこと");

    print_msg_and_wait_for_key(
        "0番目のデバイスのGPIO[1]にSingleトリガをセットする.\n次に, Enterを押し, 2秒後にトリガがかかること.\nまた, 0番目のデバイスのGPIO[1]出力と1番目のデバイスのGPIO[1]出力が25usずれていること",
    );
    let trig_time = DcSysTime::now() + Duration::from_secs(2);
    autd.send(DebugSettings::new(|dev, gpio| match (dev.idx(), gpio) {
        (0, GPIOOut::O0) => DebugType::BaseSignal,
        (0, GPIOOut::O1) => DebugType::SysTimeEq(trig_time),
        (_, GPIOOut::O0) => DebugType::BaseSignal,
        (_, GPIOOut::O1) => DebugType::SysTimeEq(trig_time + ULTRASOUND_PERIOD),
        _ => DebugType::None,
    }))
    .await?;

    println!("Enterを押して進む...");
    std::io::stdin().read_line(&mut String::new()).unwrap();

    Ok(())
}
