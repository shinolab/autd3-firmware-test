use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

pub async fn silencer_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    autd.send(Silencer::default()).await?;
    autd.send((
        Sine::new(150.).with_sampling_config(SamplingConfig::DivisionRaw(5120 * 2)),
        Focus::new(autd.geometry.center() + 150. * Vector3::z()),
    ))
    .await?;
    print_msg_and_wait_for_key("150HzのAMが適用されていること");

    autd.send(Silencer::default() * 2).await?;
    print_msg_and_wait_for_key("ノイズが小さくなったこと");

    autd.send(Silencer::default()).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    autd.send(Silencer::default() / 2).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    autd.send(Silencer::disable()).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    autd.send(Silencer::default()).await?;
    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
    let point_num = 10;
    let radius = 30.0 * MILLIMETER;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f64 / point_num as f64;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            ControlPoint::new(center + p).with_intensity(0xFF)
        })
    };
    let stm = FocusSTM::from_freq(50.).add_foci_from_iter(gen_foci());
    autd.send(stm).await?;
    print_msg_and_wait_for_key("50HzのSTMが適用されていること");

    autd.send(Silencer::default() * 2).await?;
    print_msg_and_wait_for_key("ノイズが小さくなったこと");

    autd.send(Silencer::default()).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    autd.send(Silencer::default() / 2).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    autd.send(Silencer::disable()).await?;
    print_msg_and_wait_for_key("ノイズが大きくなったこと");

    // Modulation異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::fixed_completion_steps(10, 40)?).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                Sine::with_freq_nearest(100.)
                    .with_sampling_config(SamplingConfig::DivisionRaw(5120))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                Sine::with_freq_nearest(100.)
                    .with_sampling_config(SamplingConfig::DivisionRaw(5119))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                Sine::with_freq_nearest(100.)
                    .with_sampling_config(SamplingConfig::DivisionRaw(5119))
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send(Static::new()).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                Sine::with_freq_nearest(100.)
                    .with_sampling_config(SamplingConfig::DivisionRaw(5120))
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send(Silencer::fixed_completion_steps(20, 40)?).await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::modulation(
                Segment::S1,
                TransitionMode::Immidiate
            ))
            .await
        );
    }

    // FocusSTM異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::fixed_completion_steps(10, 40)?).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                FocusSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40))
                    .add_foci_from_iter(
                        (0..2).map(|_| ControlPoint::new(Vector3::zeros()).with_intensity(0x00))
                    )
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                FocusSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40 - 1))
                    .add_foci_from_iter(
                        (0..2).map(|_| ControlPoint::new(Vector3::zeros()).with_intensity(0x00))
                    )
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                FocusSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40 - 1))
                    .add_foci_from_iter(
                        (0..2).map(|_| ControlPoint::new(Vector3::zeros()).with_intensity(0x00))
                    )
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send((Static::new(), Null::new())).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                FocusSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40))
                    .add_foci_from_iter(
                        (0..2).map(|_| ControlPoint::new(Vector3::zeros()).with_intensity(0x00))
                    )
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send(Silencer::fixed_completion_steps(20, 80)?).await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::focus_stm(
                Segment::S1,
                TransitionMode::Immidiate
            ))
            .await
        );
    }

    // GainSTM異常系
    {
        autd.send((Static::new(), Null::new())).await?;
        autd.send(Silencer::fixed_completion_steps(10, 40)?).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                GainSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40))
                    .add_gains_from_iter((0..2).map(|_| Null::new()))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                GainSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40 - 1))
                    .add_gains_from_iter((0..2).map(|_| Null::new()))
            )
            .await
        );
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(
                GainSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40 - 1))
                    .add_gains_from_iter((0..2).map(|_| Null::new()))
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send((Static::new(), Null::new())).await?;
        assert_eq!(
            Ok(true),
            autd.send(
                GainSTM::from_sampling_config(SamplingConfig::DivisionRaw(512 * 40))
                    .add_gains_from_iter((0..2).map(|_| Null::new()))
                    .with_segment(Segment::S1, None)
            )
            .await
        );
        autd.send(Silencer::fixed_completion_steps(20, 80)?).await?;
        assert_eq!(
            Err(AUTDError::Internal(
                AUTDInternalError::InvalidSilencerSettings
            )),
            autd.send(SwapSegment::gain_stm(
                Segment::S1,
                TransitionMode::Immidiate
            ))
            .await
        );
    }

    autd.send(Silencer::default()).await?;

    Ok(())
}
