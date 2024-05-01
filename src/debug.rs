use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn debug_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(ConfigureDebugSettings::new(|_| {
        [
            DebugType::BaseSignal,
            DebugType::None,
            DebugType::None,
            DebugType::None,
        ]
    }))
    .await?;

    autd.send((
        Static::new(),
        Custom::new(|dev| {
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
    print_msg_and_wait_for_key("Check that there are no outputs of GPIO[1] pins.");

    autd.send(ConfigureDebugSettings::new(|dev| {
        [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[0]),
            DebugType::None,
            DebugType::None,
        ]
    }))
    .await?;
    print_msg_and_wait_for_key("Check that a 40kHz square wave with a duty ratio of 50% are output to the GPIO[1] pins and that the phase is shifted by half a cycle.");

    autd.send(ConfigureDebugSettings::new(|dev| {
        [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[248]),
            DebugType::None,
            DebugType::None,
        ]
    }))
    .await?;
    print_msg_and_wait_for_key("Check that a 40kHz square wave with a duty ratio of about 17% are output to the GPIO[1] pins and that the phase is shifted by half a cycle.");

    autd.send(ConfigureDebugSettings::new(|dev| match dev.idx() {
        0 => [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[0]),
            DebugType::None,
            DebugType::None,
        ],
        1 => [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[248]),
            DebugType::None,
            DebugType::None,
        ],
        _ => [
            DebugType::None,
            DebugType::None,
            DebugType::None,
            DebugType::None,
        ],
    }))
    .await?;
    print_msg_and_wait_for_key("Check that a 40 kHz square wave are output on the GPIO[1] pins and that their phase are aligned.");

    autd.send(ConfigureDebugSettings::new(|dev| match dev.idx() {
        0 => [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[1]),
            DebugType::None,
            DebugType::None,
        ],
        1 => [
            DebugType::BaseSignal,
            DebugType::PwmOut(&dev[2]),
            DebugType::None,
            DebugType::None,
        ],
        _ => [
            DebugType::None,
            DebugType::None,
            DebugType::None,
            DebugType::None,
        ],
    }))
    .await?;
    print_msg_and_wait_for_key("Check that there are no outputs of GPIO[1] pins.");

    Ok(())
}
