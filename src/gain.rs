use crate::print_msg_and_wait_for_key;

use autd3::{driver::link::Link, prelude::*};

pub async fn gain_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150. * Hz),
        Focus::new(autd.geometry.center() + 150. * Vector3::z()),
    ))
    .await?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");

    autd.send(Null::new().with_segment(Segment::S1, true))
        .await?;
    print_msg_and_wait_for_key("焦点が消えたこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S1), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Gain(Segment::S0)).await?;
    print_msg_and_wait_for_key("焦点が再び提示されたこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Null::new().with_segment(Segment::S1, false))
        .await?;
    print_msg_and_wait_for_key("焦点がまだ出ていること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Gain(Segment::S1)).await?;
    print_msg_and_wait_for_key("焦点が消えたこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
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
        autd.send(SwapSegment::GainSTM(Segment::S0, TransitionMode::SyncIdx))
            .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::GainSTM(Segment::S1, TransitionMode::SyncIdx))
            .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::FocusSTM(Segment::S0, TransitionMode::SyncIdx))
            .await
    );
    assert_eq!(
        Err(AUTDError::Internal(
            AUTDInternalError::InvalidSegmentTransition
        )),
        autd.send(SwapSegment::FocusSTM(Segment::S1, TransitionMode::SyncIdx))
            .await
    );

    Ok(())
}
