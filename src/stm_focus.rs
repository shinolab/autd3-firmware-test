use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn stm_focus_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(Static::new()).await?;

    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
    let point_num = 200;
    let radius = 30.0 * MILLIMETER;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f64 / point_num as f64;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            ControlPoint::new(center + p).with_intensity(0xFF)
        })
    };

    let stm = FocusSTM::from_freq(0.5).add_foci_from_iter(gen_foci());
    autd.send(stm).await?;
    print_msg_and_wait_for_key(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に0.5HzのSTMが適用されていること",
    );
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    let stm = FocusSTM::from_freq(1.).add_foci_from_iter(gen_foci());
    autd.send(stm.with_segment(Segment::S1, Some(TransitionMode::Immidiate)))
        .await?;
    print_msg_and_wait_for_key("STM周波数が1Hzに変更されたこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    autd.send(SwapSegment::focus_stm(
        Segment::S0,
        TransitionMode::Immidiate,
    ))
    .await?;
    print_msg_and_wait_for_key("STM周波数が0.5Hzに戻ったこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1] = foci[point_num - 1].with_intensity(0x00);
    let stm = FocusSTM::from_freq(0.5)
        .with_loop_behavior(LoopBehavior::once())
        .add_foci_from_iter(foci)
        .with_segment(Segment::S1, None);
    autd.send(stm).await?;
    print_msg_and_wait_for_key("何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n焦点が右端に来たときに焦点軌道が反転し, 1サイクル後に停止すること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });
    autd.send(SwapSegment::focus_stm(Segment::S1, TransitionMode::SyncIdx))
        .await?;
    print_msg_and_wait_for_key("");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidTransitionMode
        )),
        autd.send(
            FocusSTM::from_freq(0.5)
                .add_foci_from_iter(gen_foci())
                .with_segment(Segment::S1, Some(TransitionMode::SyncIdx))
        )
        .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidTransitionMode
        )),
        autd.send(
            FocusSTM::from_freq(0.5)
                .add_foci_from_iter(gen_foci())
                .with_loop_behavior(LoopBehavior::once())
                .with_segment(Segment::S0, Some(TransitionMode::Immidiate))
        )
        .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidTransitionMode
        )),
        autd.send(SwapSegment::focus_stm(Segment::S0, TransitionMode::SyncIdx))
            .await
    );

    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::gain_stm(Segment::S0, TransitionMode::SyncIdx))
            .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::gain_stm(Segment::S1, TransitionMode::SyncIdx))
            .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::gain(Segment::S0)).await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::gain(Segment::S1)).await
    );
    Ok(())
}
