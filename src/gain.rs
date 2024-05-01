use crate::print_msg_and_wait_for_key;

use autd3::prelude::*;

pub async fn gain_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150.),
        Focus::new(autd.geometry.center() + 150. * Vector3::z()),
    ))
    .await?;
    print_msg_and_wait_for_key(
            "Check that the focal points are generated 150mm directly above the center of each device by your hands."
        );

    autd.send(Null::new().with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("Check that the focal points have disappeared.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S1), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(ChangeGainSegment::new(Segment::S0)).await?;
    print_msg_and_wait_for_key("Check that the focal points are presented again.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Null::new().with_segment(Segment::S1, None))
        .await?;
    print_msg_and_wait_for_key("Check that the focal points are still presented.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(ChangeGainSegment::new(Segment::S1)).await?;
    print_msg_and_wait_for_key("Check that the focal points have disappeared.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S1), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
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
        autd.send(ChangeFocusSTMSegment::new(
            Segment::S0,
            TransitionMode::SyncIdx
        ))
        .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(ChangeFocusSTMSegment::new(
            Segment::S1,
            TransitionMode::SyncIdx
        ))
        .await
    );

    Ok(())
}
