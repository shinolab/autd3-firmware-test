use autd3::{derive::*, driver::link::Link, prelude::*};

pub async fn clear_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(Clear::new()).await?;
    autd.send(ReadsFPGAState::new(|_| true)).await?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });
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

    Ok(())
}
