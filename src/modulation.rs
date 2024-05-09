use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn modulation_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150.),
        Focus::new(autd.geometry.center() + 150. * Vector3::z()),
    ))
    .await?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Static::new().with_segment(Segment::S1, Some(TransitionMode::Immidiate)))
        .await?;
    print_msg_and_wait_for_key("AMが適用されていないこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::modulation(
        Segment::S0,
        TransitionMode::Immidiate,
    ))
    .await?;
    print_msg_and_wait_for_key("AMが再び適用されたこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Static::with_intensity(0).with_segment(Segment::S1, None))
        .await?;
    print_msg_and_wait_for_key("AMがまだ適用されていること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::modulation(
        Segment::S1,
        TransitionMode::Immidiate,
    ))
    .await?;
    print_msg_and_wait_for_key("AMが適用されていないこと");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    #[derive(Modulation, Clone, Copy)]
    pub struct Sawtooth {
        config: SamplingConfig,
        loop_behavior: LoopBehavior,
        reverse: bool,
    }

    impl Sawtooth {
        fn new() -> Self {
            Self {
                config: SamplingConfig::FreqNearest(256.),
                loop_behavior: LoopBehavior::once(),
                reverse: false,
            }
        }

        fn reverse() -> Self {
            Self {
                config: SamplingConfig::FreqNearest(256.),
                loop_behavior: LoopBehavior::once(),
                reverse: true,
            }
        }
    }

    impl Modulation for Sawtooth {
        fn calc(&self, geometry: &Geometry) -> Result<HashMap<usize, Vec<u8>>, AUTDInternalError> {
            Self::transform(geometry, |_| {
                let mut res = (0..=255u8).collect::<Vec<_>>();
                if self.reverse {
                    res.reverse();
                }
                Ok(res)
            })
        }
    }

    autd.send(Sawtooth::new().with_segment(Segment::S0, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("のこぎり波AMが1波形分だけ適用されること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(Sawtooth::reverse().with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
        .await?;
    print_msg_and_wait_for_key("逆のこぎり波AMが1波形分だけ適用されること");
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    {
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidTransitionMode
            )),
            autd.send(Static::new().with_segment(Segment::S1, Some(TransitionMode::SyncIdx)))
                .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidTransitionMode
            )),
            autd.send(
                Static::new()
                    .with_loop_behavior(LoopBehavior::once())
                    .with_segment(Segment::S0, Some(TransitionMode::Immidiate))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidTransitionMode
            )),
            autd.send(SwapSegment::modulation(
                Segment::S0,
                TransitionMode::Immidiate
            ))
            .await
        );
    }

    Ok(())
}
