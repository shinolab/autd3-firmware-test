use std::num::NonZeroU16;

use crate::print_msg_and_wait_for_key;

use autd3::{
    core::common::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
    core::{common::ULTRASOUND_PERIOD, link::Link},
    prelude::*,
};

pub fn silencer_test<L: Link>(autd: &mut Controller<L, firmware::V12_1>) -> anyhow::Result<()> {
    // Modulation
    {
        autd.send(Silencer::default())?;
        autd.send((
            Sine::new(
                150. * Hz,
                SineOption {
                    sampling_config: SamplingConfig::new(NonZeroU16::new(20).unwrap()),
                    ..Default::default()
                },
            ),
            Focus::new(
                autd.geometry().center() + 150. * Vector3::z(),
                Default::default(),
            ),
        ))?;
        print_msg_and_wait_for_key("150HzのAMが適用されていること");

        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT as u32 * 2,
            phase: ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT as u32 * 2,
            strict: true,
        }))?;
        print_msg_and_wait_for_key("ノイズが小さくなったこと");

        autd.send(Silencer::default())?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT as u32 / 2,
            phase: ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT as u32 / 2,
            strict: true,
        }))?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::disable())?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");
    }

    // STM
    {
        autd.send(Silencer::default())?;
        let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
        let point_num = 10;
        let radius = 30.0 * mm;
        let gen_foci = || {
            (0..point_num).map(|i| {
                let theta = 2.0 * PI * i as f32 / point_num as f32;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                ControlPoint::new(center + p, Phase::ZERO)
            })
        };
        let stm = FociSTM::new(gen_foci().collect::<Vec<_>>(), 50. * Hz);
        autd.send(stm)?;
        print_msg_and_wait_for_key("50HzのSTMが適用されていること");

        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT as u32 * 2,
            phase: ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT as u32 * 2,
            strict: true,
        }))?;
        print_msg_and_wait_for_key("ノイズが小さくなったこと");

        autd.send(Silencer::default())?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT as u32 / 2,
            phase: ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT as u32 / 2,
            strict: true,
        }))?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::disable())?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");
    }

    // Modulation異常系
    {
        autd.send((Static::default(), Null::new()))?;
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 10,
            phase: ULTRASOUND_PERIOD * 40,
            strict: true,
        }))?;
        assert!(
            autd.send(
                Sine::new(
                    100. * Hz,
                    SineOption {
                        sampling_config: SamplingConfig::new(NonZeroU16::new(10).unwrap()),
                        ..Default::default()
                    }
                )
                .into_nearest()
            )
            .is_ok()
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(
                Sine::new(
                    100. * Hz,
                    SineOption {
                        sampling_config: SamplingConfig::new(NonZeroU16::new(9).unwrap()),
                        ..Default::default()
                    }
                )
                .into_nearest()
            )
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(WithSegment {
                inner: Sine::new(
                    100. * Hz,
                    SineOption {
                        sampling_config: SamplingConfig::new(NonZeroU16::new(9).unwrap()),
                        ..Default::default()
                    }
                )
                .into_nearest(),
                segment: Segment::S1,
                transition_mode: None,
            })
        );
        autd.send(Static::default())?;
        assert!(
            autd.send(WithSegment {
                inner: Sine::new(
                    100. * Hz,
                    SineOption {
                        sampling_config: SamplingConfig::new(NonZeroU16::new(10).unwrap()),
                        ..Default::default()
                    }
                )
                .into_nearest(),
                segment: Segment::S1,
                transition_mode: None,
            })
            .is_ok()
        );
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 20,
            phase: ULTRASOUND_PERIOD * 40,
            strict: true,
        }))?;
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(SwapSegment::Modulation(
                Segment::S1,
                TransitionMode::Immediate
            ))
        );
    }

    // FociSTM異常系
    {
        autd.send((Static::default(), Null::new()))?;
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 10,
            phase: ULTRASOUND_PERIOD * 40,
            strict: true,
        }))?;
        assert!(
            autd.send(FociSTM::new(
                (0..2).map(|_| ControlPoint::default()).collect::<Vec<_>>(),
                SamplingConfig::new(NonZeroU16::new(40).unwrap()),
            ))
            .is_ok()
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(FociSTM::new(
                (0..2).map(|_| ControlPoint::default()).collect::<Vec<_>>(),
                SamplingConfig::new(NonZeroU16::new(39).unwrap()),
            ))
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(WithSegment {
                inner: FociSTM::new(
                    (0..2).map(|_| ControlPoint::default()).collect::<Vec<_>>(),
                    SamplingConfig::new(NonZeroU16::new(39).unwrap()),
                ),
                segment: Segment::S1,
                transition_mode: None,
            })
        );
        autd.send((Static::default(), Null::new()))?;
        assert!(
            autd.send(WithSegment {
                inner: FociSTM::new(
                    (0..2).map(|_| ControlPoint::default()).collect::<Vec<_>>(),
                    SamplingConfig::new(NonZeroU16::new(40).unwrap()),
                ),
                segment: Segment::S1,
                transition_mode: None,
            })
            .is_ok()
        );
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 20,
            phase: ULTRASOUND_PERIOD * 80,
            strict: true,
        }))?;
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(SwapSegment::FociSTM(Segment::S1, TransitionMode::Immediate))
        );
    }

    // GainSTM異常系
    {
        autd.send((Static::default(), Null::new()))?;
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 10,
            phase: ULTRASOUND_PERIOD * 40,
            strict: true,
        }))?;
        assert!(
            autd.send(GainSTM::new(
                (0..2).map(|_| Null::new()).collect::<Vec<_>>(),
                SamplingConfig::new(NonZeroU16::new(40).unwrap()),
                Default::default(),
            ))
            .is_ok()
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(GainSTM::new(
                (0..2).map(|_| Null::new()).collect::<Vec<_>>(),
                SamplingConfig::new(NonZeroU16::new(39).unwrap()),
                Default::default(),
            ))
        );
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(WithSegment {
                inner: GainSTM::new(
                    (0..2).map(|_| Null::new()).collect::<Vec<_>>(),
                    SamplingConfig::new(NonZeroU16::new(39).unwrap()),
                    Default::default(),
                ),
                segment: Segment::S1,
                transition_mode: None,
            })
        );
        autd.send((Static::default(), Null::new()))?;
        assert!(
            autd.send(WithSegment {
                inner: GainSTM::new(
                    (0..2).map(|_| Null::new()).collect::<Vec<_>>(),
                    SamplingConfig::new(NonZeroU16::new(40).unwrap()),
                    Default::default(),
                ),
                segment: Segment::S1,
                transition_mode: None,
            })
            .is_ok()
        );
        autd.send(Silencer::new(FixedCompletionTime {
            intensity: ULTRASOUND_PERIOD * 20,
            phase: ULTRASOUND_PERIOD * 80,
            strict: true,
        }))?;
        assert_eq!(
            Err(AUTDDriverError::InvalidSilencerSettings),
            autd.send(SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate))
        );
    }

    autd.send(Silencer::default())?;

    Ok(())
}
