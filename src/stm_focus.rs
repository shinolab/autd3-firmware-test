use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, driver::firmware::v12_1::fpga::FOCI_STM_BUF_SIZE_MAX, prelude::*};

pub fn stm_focus_test<L: Link>(autd: &mut Controller<L, firmware::V12_1>) -> anyhow::Result<()> {
    autd.send(Static::default())?;
    autd.send(Silencer::disable())?;

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
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    let stm = FociSTM::new(gen_foci().collect::<Vec<_>>(), 1.0 * Hz);
    autd.send(WithSegment {
        inner: stm,
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Immediate),
    })?;
    print_msg_and_wait_for_key("STM周波数が1Hzに変更されたこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    autd.send(SwapSegment::FociSTM(Segment::S0, TransitionMode::Immediate))?;
    print_msg_and_wait_for_key("STM周波数が0.5Hzに戻ったこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1].intensity = Intensity::MIN;
    let stm = WithLoopBehavior {
        inner: FociSTM::new(foci, 0.5 * Hz),
        loop_behavior: LoopBehavior::ONCE,
        segment: Segment::S1,
        transition_mode: None,
    };
    autd.send(stm)?;
    print_msg_and_wait_for_key(
        "何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n焦点が右端に来たときに焦点軌道が反転し, 1サイクル後に停止すること",
    );
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });
    autd.send(SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx))?;
    print_msg_and_wait_for_key("");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    let stm = FociSTM::new(
        (0..FOCI_STM_BUF_SIZE_MAX / 8)
            .map(|i| {
                let theta = 2.0 * PI * i as f32 / (FOCI_STM_BUF_SIZE_MAX / 8) as f32;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                [
                    ControlPoint::new(center + p, Phase::ZERO),
                    ControlPoint::new(center - p, Phase::ZERO),
                    ControlPoint::new(center + p, Phase::ZERO),
                    ControlPoint::new(center - p, Phase::ZERO),
                    ControlPoint::new(center + p, Phase::ZERO),
                    ControlPoint::new(center - p, Phase::ZERO),
                    ControlPoint::new(center + p, Phase::ZERO),
                    ControlPoint::new(center - p, Phase::ZERO),
                ]
            })
            .collect::<Vec<_>>(),
        SamplingConfig::FREQ_4K,
    );
    autd.send((
        Sine {
            freq: 150 * Hz,
            option: SineOption::default(),
        },
        stm,
    ))?;
    print_msg_and_wait_for_key(&format!(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に2焦点{}HzのSTMが適用されていること",
        4_000.0 / (FOCI_STM_BUF_SIZE_MAX / 8) as f32
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    autd.send(WithSegment {
        inner: FociSTM {
            foci: Circle {
                center,
                radius,
                num_points: FOCI_STM_BUF_SIZE_MAX,
                n: Vector3::z_axis(),
                intensity: Intensity::MAX,
            },
            config: SamplingConfig::FREQ_40K,
        },
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Immediate),
    })?;
    print_msg_and_wait_for_key(&format!(
        "周波数{}HzのSTMが適用されていること",
        40_000.0 / FOCI_STM_BUF_SIZE_MAX as f32
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    assert_eq!(
        Err(AUTDDriverError::InvalidTransitionMode),
        autd.send(WithSegment {
            inner: FociSTM::new(gen_foci().collect::<Vec<_>>(), 0.5 * Hz),
            segment: Segment::S1,
            transition_mode: Some(TransitionMode::SyncIdx),
        })
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidTransitionMode),
        autd.send(WithLoopBehavior {
            inner: FociSTM::new(gen_foci().collect::<Vec<_>>(), 0.5 * Hz),
            loop_behavior: LoopBehavior::ONCE,
            segment: Segment::S0,
            transition_mode: Some(TransitionMode::Immediate),
        })
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidTransitionMode),
        autd.send(SwapSegment::FociSTM(Segment::S0, TransitionMode::SyncIdx))
    );

    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::GainSTM(Segment::S0, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::GainSTM(Segment::S1, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::Gain(Segment::S0, TransitionMode::Immediate))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::Gain(Segment::S1, TransitionMode::Immediate))
    );

    autd.send(Clear {})?;

    Ok(())
}
