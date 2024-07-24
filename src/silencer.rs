use std::num::NonZeroU16;

use crate::print_msg_and_wait_for_key;

use autd3::{
    derive::*,
    driver::{
        defined::ControlPoint,
        firmware::fpga::{SILENCER_STEPS_INTENSITY_DEFAULT, SILENCER_STEPS_PHASE_DEFAULT},
        link::Link,
    },
    prelude::*,
};

pub async fn silencer_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    // Modulation
    {
        autd.send(Silencer::default()).await?;
        autd.send((
            Sine::new(150. * Hz)
                .with_sampling_config(SamplingConfig::Division(NonZeroU16::new(20).unwrap())),
            Focus::new(autd.geometry().center() + 150. * Vector3::z()),
        ))
        .await?;
        print_msg_and_wait_for_key("150HzのAMが適用されていること");

        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT * 2,
            ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT * 2,
        ))
        .await?;
        print_msg_and_wait_for_key("ノイズが小さくなったこと");

        autd.send(Silencer::default()).await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT / 2,
            ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT / 2,
        ))
        .await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::disable()).await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");
    }

    // STM
    {
        autd.send(Silencer::default()).await?;
        let center = autd.geometry().center() + Vector3::new(0., 0., 150.0 * mm);
        let point_num = 10;
        let radius = 30.0 * mm;
        let gen_foci = || {
            (0..point_num).map(|i| {
                let theta = 2.0 * PI * i as f32 / point_num as f32;
                let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
                ControlPoint::new(center + p)
            })
        };
        let stm = FociSTM::new(50. * Hz, gen_foci())?;
        autd.send(stm).await?;
        print_msg_and_wait_for_key("50HzのSTMが適用されていること");

        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT * 2,
            ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT * 2,
        ))
        .await?;
        print_msg_and_wait_for_key("ノイズが小さくなったこと");

        autd.send(Silencer::default()).await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * SILENCER_STEPS_INTENSITY_DEFAULT / 2,
            ULTRASOUND_PERIOD * SILENCER_STEPS_PHASE_DEFAULT / 2,
        ))
        .await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");

        autd.send(Silencer::disable()).await?;
        print_msg_and_wait_for_key("ノイズが大きくなったこと");
    }

    // Modulation異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 10,
            ULTRASOUND_PERIOD * 40,
        ))
        .await?;
        assert!(autd
            .send(
                Sine::from_freq_nearest(100. * Hz)
                    .with_sampling_config(SamplingConfig::Division(NonZeroU16::new(10).unwrap()))
            )
            .await
            .is_ok());
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                Sine::from_freq_nearest(100. * Hz)
                    .with_sampling_config(SamplingConfig::Division(NonZeroU16::new(9).unwrap()))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                Sine::from_freq_nearest(100. * Hz)
                    .with_sampling_config(SamplingConfig::Division(NonZeroU16::new(9).unwrap()))
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send(Static::new()).await?;
        assert!(autd
            .send(
                Sine::from_freq_nearest(100. * Hz)
                    .with_sampling_config(SamplingConfig::Division(NonZeroU16::new(10).unwrap()))
                    .with_segment(Segment::S1, None)
            )
            .await
            .is_ok());
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 20,
            ULTRASOUND_PERIOD * 40,
        ))
        .await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::Modulation(
                Segment::S1,
                TransitionMode::Immediate
            ))
            .await
        );
    }

    // FociSTM異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 10,
            ULTRASOUND_PERIOD * 40,
        ))
        .await?;
        assert!(autd
            .send(FociSTM::new(
                SamplingConfig::Division(NonZeroU16::new(40).unwrap()),
                (0..2).map(|_| (ControlPoint::new(Vector3::zeros()), 0x00))
            )?)
            .await
            .is_ok());
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(FociSTM::new(
                SamplingConfig::Division(NonZeroU16::new(39).unwrap()),
                (0..2).map(|_| (ControlPoint::new(Vector3::zeros()), 0x00))
            )?)
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                FociSTM::new(
                    SamplingConfig::Division(NonZeroU16::new(39).unwrap()),
                    (0..2).map(|_| (ControlPoint::new(Vector3::zeros()), 0x00))
                )?
                .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send((Static::new(), Null::new())).await?;
        assert!(autd
            .send(
                FociSTM::new(
                    SamplingConfig::Division(NonZeroU16::new(40).unwrap()),
                    (0..2).map(|_| (ControlPoint::new(Vector3::zeros()), 0x00))
                )?
                .with_segment(Segment::S1, None)
            )
            .await
            .is_ok());
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 20,
            ULTRASOUND_PERIOD * 80,
        ))
        .await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::FociSTM(Segment::S1, TransitionMode::Immediate))
                .await
        );
    }

    // GainSTM異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 10,
            ULTRASOUND_PERIOD * 40,
        ))
        .await?;
        assert!(autd
            .send(GainSTM::new(
                SamplingConfig::Division(NonZeroU16::new(40).unwrap()),
                (0..2).map(|_| Null::new())
            )?)
            .await
            .is_ok());
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(GainSTM::new(
                SamplingConfig::Division(NonZeroU16::new(39).unwrap()),
                (0..2).map(|_| Null::new())
            )?)
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                GainSTM::new(
                    SamplingConfig::Division(NonZeroU16::new(39).unwrap()),
                    (0..2).map(|_| Null::new())
                )?
                .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send((Static::new(), Null::new())).await?;
        assert!(autd
            .send(
                GainSTM::new(
                    SamplingConfig::Division(NonZeroU16::new(40).unwrap()),
                    (0..2).map(|_| Null::new())
                )?
                .with_segment(Segment::S1, None)
            )
            .await
            .is_ok());
        autd.send(Silencer::from_completion_time(
            ULTRASOUND_PERIOD * 20,
            ULTRASOUND_PERIOD * 80,
        ))
        .await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::GainSTM(Segment::S1, TransitionMode::Immediate))
                .await
        );
    }

    autd.send(Silencer::default()).await?;

    Ok(())
}
