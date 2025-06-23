use std::time::Duration;

use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, driver::datagram::EmulateGPIOIn, prelude::*};

fn transition_test_focus_stm<L: Link>(
    autd: &mut Controller<L, firmware::Latest>,
) -> anyhow::Result<()> {
    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
    let point_num = 200;
    let radius = 30.0 * mm;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f32 / point_num as f32;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            ControlPoints::<1>::from(ControlPoint::new(center + p, Phase::ZERO))
        })
    };

    let stm = FociSTM::new(gen_foci().collect::<Vec<_>>(), 0.5 * Hz);
    autd.send(stm)?;
    print_msg_and_wait_for_key(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に0.5HzのSTMが適用されていること",
    );

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1] = ControlPoints::<1> {
        intensity: Intensity::MIN,
        ..foci[point_num - 1].clone()
    };
    let stm = WithLoopBehavior {
        inner: FociSTM::new(foci, 0.5 * Hz),
        loop_behavior: LoopBehavior::ONCE,
        segment: Segment::S1,
        transition_mode: None,
    };
    autd.send(stm)?;
    print_msg_and_wait_for_key(
        "何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n2秒後(焦点が再び左端に来た時)に焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること",
    );
    autd.send(SwapSegment::FociSTM(
        Segment::S1,
        TransitionMode::SysTime(DcSysTime::now() + Duration::from_millis(2000)),
    ))?;
    print_msg_and_wait_for_key("");

    autd.send(SwapSegment::FociSTM(Segment::S0, TransitionMode::Immediate))?;
    print_msg_and_wait_for_key("再び0.5HzのSTMが適用されたこと");

    print_msg_and_wait_for_key(
        "焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n直ちに焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること",
    );
    autd.send((
        SwapSegment::FociSTM(Segment::S1, TransitionMode::GPIO(GPIOIn::I0)),
        EmulateGPIOIn::new(|_| |gpio| gpio == GPIOIn::I0),
    ))?;

    print_msg_and_wait_for_key("");

    autd.send(Sine::new(150. * Hz, Default::default()))?;
    let stm = FociSTM::new(
        vec![
            ControlPoint::new(center + Vector3::new(30., 0., 0.), Phase::ZERO),
            ControlPoint::new(center + Vector3::new(0., 30., 0.), Phase::ZERO),
        ],
        0.5 * Hz,
    );
    autd.send(stm)?;
    let stm = WithSegment {
        inner: FociSTM::new(
            vec![
                ControlPoint::new(center + Vector3::new(-30., 0., 0.), Phase::ZERO),
                ControlPoint::new(center + Vector3::new(0., -30., 0.), Phase::ZERO),
            ],
            0.5 * Hz,
        ),
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Ext),
    };
    autd.send(stm)?;
    print_msg_and_wait_for_key("1秒ごとに焦点が正方形の頂点にジャンプすること");

    {
        autd.send((Static::default(), Null::new()))?;
        let stm = WithLoopBehavior {
            inner: FociSTM::new(
                (0..2)
                    .map(|_| ControlPoint::new(Point3::origin(), Phase::ZERO))
                    .collect::<Vec<_>>(),
                0.5 * Hz,
            ),
            loop_behavior: LoopBehavior::ONCE,
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::SysTime(DcSysTime::now())),
        };
        assert_eq!(Err(AUTDDriverError::MissTransitionTime), autd.send(stm));
    }

    Ok(())
}

fn transition_test_gain_stm<L: Link>(
    autd: &mut Controller<L, firmware::Latest>,
) -> anyhow::Result<()> {
    let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
    let point_num = 200;
    let radius = 30.0 * mm;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f32 / point_num as f32;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            Focus::new(center + p, Default::default())
        })
    };

    let stm = GainSTM::new(gen_foci().collect::<Vec<_>>(), 0.5 * Hz, Default::default());
    autd.send(stm)?;
    print_msg_and_wait_for_key(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に0.5HzのSTMが適用されていること",
    );

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1].option.intensity = Intensity::MIN;
    let stm = WithLoopBehavior {
        inner: GainSTM::new(foci, 0.5 * Hz, Default::default()),
        loop_behavior: LoopBehavior::ONCE,
        segment: Segment::S1,
        transition_mode: None,
    };
    autd.send(stm)?;
    print_msg_and_wait_for_key(
        "何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n2秒後(焦点が再び左端に来た時)に焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること",
    );
    autd.send(SwapSegment::GainSTM(
        Segment::S1,
        TransitionMode::SysTime(DcSysTime::now() + Duration::from_millis(2000)),
    ))?;
    print_msg_and_wait_for_key("");

    autd.send(SwapSegment::GainSTM(Segment::S0, TransitionMode::Immediate))?;
    print_msg_and_wait_for_key("再び0.5HzのSTMが適用されたこと");

    print_msg_and_wait_for_key(
        "焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n直ちに焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること",
    );
    autd.send((
        SwapSegment::GainSTM(Segment::S1, TransitionMode::GPIO(GPIOIn::I0)),
        EmulateGPIOIn::new(|_| |gpio| gpio == GPIOIn::I0),
    ))?;

    print_msg_and_wait_for_key("");

    autd.send(Sine::new(150. * Hz, Default::default()))?;
    let stm = GainSTM::new(
        vec![
            Focus::new(center + Vector3::new(30., 0., 0.), Default::default()),
            Focus::new(center + Vector3::new(0., 30., 0.), Default::default()),
        ],
        0.5 * Hz,
        Default::default(),
    );
    autd.send(stm)?;
    let stm = WithSegment {
        inner: GainSTM::new(
            vec![
                Focus::new(center + Vector3::new(-30., 0., 0.), Default::default()),
                Focus::new(center + Vector3::new(0., -30., 0.), Default::default()),
            ],
            0.5 * Hz,
            Default::default(),
        ),
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Ext),
    };
    autd.send(stm)?;
    print_msg_and_wait_for_key("1秒ごとに焦点が正方形の頂点にジャンプすること");

    {
        autd.send((Static::default(), Null::new()))?;
        let stm = WithLoopBehavior {
            inner: GainSTM::new(
                (0..2).map(|_| Null::new()).collect::<Vec<_>>(),
                0.5 * Hz,
                Default::default(),
            ),
            loop_behavior: LoopBehavior::ONCE,
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::SysTime(DcSysTime::now())),
        };
        assert_eq!(Err(AUTDDriverError::MissTransitionTime), autd.send(stm));
    }

    Ok(())
}

pub fn transition_test<L: Link>(autd: &mut Controller<L, firmware::Latest>) -> anyhow::Result<()> {
    transition_test_focus_stm(autd)?;
    transition_test_gain_stm(autd)?;

    Ok(())
}
