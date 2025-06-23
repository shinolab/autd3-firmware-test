use crate::print_msg_and_wait_for_key;

use autd3::{
    core::{derive::*, link::Link},
    driver::firmware::latest::fpga::MOD_BUF_SIZE_MAX,
    prelude::*,
};

pub fn modulation_test<L: Link>(autd: &mut Controller<L, firmware::Latest>) -> anyhow::Result<()> {
    autd.send((
        Sine::new(150. * Hz, Default::default()),
        Focus::new(
            autd.geometry().center() + 150. * Vector3::z(),
            Default::default(),
        ),
    ))?;
    print_msg_and_wait_for_key("各デバイスの中心から150mm直上に焦点が生成されていること");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(WithSegment {
        inner: Static::default(),
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::Immediate),
    })?;
    print_msg_and_wait_for_key("AMが適用されていないこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Modulation(
        Segment::S0,
        TransitionMode::Immediate,
    ))?;
    print_msg_and_wait_for_key("AMが再び適用されたこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(WithSegment {
        inner: Static::new(0x00),
        segment: Segment::S1,
        transition_mode: None,
    })?;
    print_msg_and_wait_for_key("AMがまだ適用されていること");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Modulation(
        Segment::S1,
        TransitionMode::Immediate,
    ))?;
    print_msg_and_wait_for_key("AMが適用されていないこと");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send((
        autd3::modulation::Custom {
            buffer: std::iter::repeat([vec![0xFF; 1], vec![0; MOD_BUF_SIZE_MAX / 2 - 1]].concat())
                .take(2)
                .flatten()
                .collect(),
            sampling_config: SamplingConfig::FREQ_4K,
        },
        Focus::new(
            autd.geometry().center() + 150. * Vector3::z(),
            Default::default(),
        ),
    ))?;
    print_msg_and_wait_for_key(&format!(
        "{:?}に1回, 単発音が聞こえること",
        std::time::Duration::from_micros(250) * MOD_BUF_SIZE_MAX as u32 / 2
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send((
        autd3::modulation::Custom {
            buffer: std::iter::repeat([vec![0; MOD_BUF_SIZE_MAX / 2 - 1], vec![0xFF; 1]].concat())
                .take(2)
                .flatten()
                .collect(),
            sampling_config: SamplingConfig::FREQ_4K,
        },
        Focus::new(
            autd.geometry().center() + 150. * Vector3::z(),
            Default::default(),
        ),
    ))?;
    print_msg_and_wait_for_key(&format!(
        "{:?}に1回, 単発音が聞こえること",
        std::time::Duration::from_micros(250) * MOD_BUF_SIZE_MAX as u32 / 2
    ));
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(SwapSegment::Modulation(
        Segment::S1,
        TransitionMode::Immediate,
    ))?;
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    #[derive(Modulation, Clone, Copy, Debug)]
    pub struct Sawtooth {
        config: SamplingConfig,
        reverse: bool,
    }

    impl Sawtooth {
        fn new() -> Self {
            Self {
                config: SamplingConfig::new(256. * Hz).into_nearest(),
                reverse: false,
            }
        }

        fn reverse() -> Self {
            Self {
                config: SamplingConfig::new(256. * Hz).into_nearest(),
                reverse: true,
            }
        }
    }

    impl Modulation for Sawtooth {
        fn calc(self, _limits: &FirmwareLimits) -> Result<Vec<u8>, ModulationError> {
            let mut res = (0..=255u8).collect::<Vec<_>>();
            if self.reverse {
                res.reverse();
            }
            Ok(res)
        }

        fn sampling_config(&self) -> SamplingConfig {
            self.config
        }
    }

    autd.send(WithLoopBehavior {
        inner: Sawtooth::new(),
        loop_behavior: LoopBehavior::ONCE,
        segment: Segment::S0,
        transition_mode: Some(TransitionMode::SyncIdx),
    })?;
    print_msg_and_wait_for_key("のこぎり波AMが1波形分だけ適用されること");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    autd.send(WithLoopBehavior {
        inner: Sawtooth::reverse(),
        loop_behavior: LoopBehavior::ONCE,
        segment: Segment::S1,
        transition_mode: Some(TransitionMode::SyncIdx),
    })?;
    print_msg_and_wait_for_key("逆のこぎり波AMが1波形分だけ適用されること");
    std::thread::sleep(std::time::Duration::from_millis(100));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S1, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    {
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            autd.send(WithSegment {
                inner: Static::default(),
                segment: Segment::S1,
                transition_mode: Some(TransitionMode::SyncIdx),
            })
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            autd.send(WithLoopBehavior {
                inner: Static::default(),
                loop_behavior: LoopBehavior::ONCE,
                segment: Segment::S0,
                transition_mode: Some(TransitionMode::Immediate),
            })
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidTransitionMode),
            autd.send(SwapSegment::Modulation(
                Segment::S0,
                TransitionMode::Immediate
            ))
        );
    }

    Ok(())
}
