use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, prelude::*};

pub fn output_mask_test<L: Link>(autd: &mut Controller<L, firmware::V12_1>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150. * Hz, SineOption::default()),
        Focus::new(
            autd.geometry().center() + 150. * Vector3::z(),
            FocusOption::default(),
        ),
    ))?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");

    autd.send(OutputMask::new(
        |dev| {
            let dev_idx = dev.idx();
            let dev_center = dev.center().x;
            move |tr| match dev_idx {
                0 => tr.position().x < dev_center / 2.0,
                1 => tr.position().x >= dev_center / 2.0,
                _ => true,
            }
        },
        Segment::S0,
    ))?;
    print_msg_and_wait_for_key(
        "0番目のデバイスの左半分, 1番めのデバイスの右半分だけが出力していること",
    );

    autd.send(PhaseCorrection::new(|_dev| |_tr| Phase(0)))?;
    autd.send(Static::default())?;
    Ok(())
}
