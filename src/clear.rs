use autd3::{core::link::Link, prelude::*};

pub fn clear_test<L: Link>(autd: &mut Controller<L, firmware::Latest>) -> anyhow::Result<()> {
    autd.send(Clear::new())?;
    autd.send(ReadsFPGAState::new(|_| true))?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::FociSTM(Segment::S0, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::GainSTM(Segment::S0, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::GainSTM(Segment::S1, TransitionMode::SyncIdx))
    );

    Ok(())
}
