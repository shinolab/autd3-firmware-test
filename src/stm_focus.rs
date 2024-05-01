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
    "Check that the focal points are moving at a freq of 0.5 Hz over a circumference of 30 mm radius by your hands."
);
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });

    let stm = FocusSTM::from_freq(1.).add_foci_from_iter(gen_foci());
    autd.send(stm.with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("Check that the freq is now 1 Hz.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    autd.send(ChangeFocusSTMSegment::new(
        Segment::S0,
        TransitionMode::SyncIdx,
    ))
    .await?;
    print_msg_and_wait_for_key("Check that the freq returned to 0.5 Hz.");
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
    print_msg_and_wait_for_key("Check that the nothing has chenged. Then, continue if the focal point is on the left size of device and check that the focus movement direction reverses when the focus comes to the right edge and stops after a cycle.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S0), state.current_stm_segment());
    });
    autd.send(ChangeFocusSTMSegment::new(
        Segment::S1,
        TransitionMode::SyncIdx,
    ))
    .await?;
    print_msg_and_wait_for_key("");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(None, state.current_gain_segment());
        assert_eq!(Some(Segment::S1), state.current_stm_segment());
    });

    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(ChangeGainSTMSegment::new(
            Segment::S0,
            TransitionMode::SyncIdx
        ))
        .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(ChangeGainSTMSegment::new(
            Segment::S1,
            TransitionMode::SyncIdx
        ))
        .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(ChangeGainSegment::new(Segment::S0)).await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(ChangeGainSegment::new(Segment::S1)).await
    );
    Ok(())
}
