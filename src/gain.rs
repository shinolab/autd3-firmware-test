use crate::print_msg_and_wait_for_key;

use autd3::{core::link::Link, prelude::*};

pub fn gain_test<L: Link>(autd: &mut Controller<L, firmware::Latest>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150. * Hz, SineOption::default()),
        Focus::new(
            autd.geometry().center() + 150. * Vector3::z(),
            FocusOption::default(),
        ),
    ))?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");

    autd.send(WithSegment {
        inner: Null::new(),
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Immediate),
    })?;
    print_msg_and_wait_for_key("焦点が消えたこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S1), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Gain(Segment::S0, TransitionMode::Immediate))?;
    print_msg_and_wait_for_key("焦点が再び提示されたこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(WithSegment {
        inner: Null::new(),
        segment: Segment::S1,
        transition_mode: None,
    })?;
    print_msg_and_wait_for_key("焦点がまだ出ていること");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Gain(Segment::S1, TransitionMode::Immediate))?;
    print_msg_and_wait_for_key("焦点が消えたこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S1), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

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
        autd.send(SwapSegment::FociSTM(Segment::S0, TransitionMode::SyncIdx))
    );
    assert_eq!(
        Err(AUTDDriverError::InvalidSegmentTransition),
        autd.send(SwapSegment::FociSTM(Segment::S1, TransitionMode::SyncIdx))
    );

    Ok(())
}
