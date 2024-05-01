use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn modulation_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150.),
        Focus::new(autd.geometry.center() + 150. * Vector3::z()),
    ))
    .await?;
    print_msg_and_wait_for_key(
        "Check that the focal points are generated 150mm directly above the center of each device by your hands."
    );
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Static::new().with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("Check that the AM modulation is no longer applied.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(ChangeModulationSegment::new(
        Segment::S0,
        TransitionMode::SyncIdx,
    ))
    .await?;
    print_msg_and_wait_for_key("Check that the AM modulation has been applied again.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Static::with_intensity(0).with_segment(Segment::S1, None))
        .await?;
    print_msg_and_wait_for_key("Check that the focal points are still presented.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(ChangeModulationSegment::new(
        Segment::S1,
        TransitionMode::SyncIdx,
    ))
    .await?;
    print_msg_and_wait_for_key("Check that the focal points have disappeared.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    #[derive(Modulation, Clone, Copy)]
    pub struct Sawtooth {
        config: SamplingConfiguration,
        loop_behavior: LoopBehavior,
        reverse: bool,
    }

    impl Sawtooth {
        fn new() -> Self {
            Self {
                config: SamplingConfiguration::from_freq_nearest(256.).unwrap(),
                loop_behavior: LoopBehavior::once(),
                reverse: false,
            }
        }

        fn reverse() -> Self {
            Self {
                config: SamplingConfiguration::from_freq_nearest(256.).unwrap(),
                loop_behavior: LoopBehavior::once(),
                reverse: true,
            }
        }
    }

    impl Modulation for Sawtooth {
        fn calc(&self) -> Result<Vec<EmitIntensity>, AUTDInternalError> {
            let mut res = (0..=255u8)
                .map(|i| EmitIntensity::new(i))
                .collect::<Vec<_>>();
            if self.reverse {
                res.reverse();
            }
            Ok(res)
        }
    }

    autd.send(Sawtooth::new().with_segment(Segment::S0, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("Check that the AM modulation is applied with a sawtooth pattern.");
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Sawtooth::reverse().with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key(
        "Check that the AM modulation is applied with a reversed sawtooth pattern.",
    );
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    Ok(())
}
