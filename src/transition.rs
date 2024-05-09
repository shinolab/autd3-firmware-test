use std::time::Duration;

use crate::print_msg_and_wait_for_key;

use autd3::{derive::*, prelude::*};

async fn transition_test_focus_stm<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
    let point_num = 200;
    let radius = 30.0 * MILLIMETER;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f64 / point_num as f64;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            ControlPoint::new(center + p).with_intensity(0xFF)
        })
    };

    let stm = FocusSTM::from_freq(0.5).add_foci_from_iter(gen_foci());
    autd.send(stm).await?;
    print_msg_and_wait_for_key(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に0.5HzのSTMが適用されていること",
    );

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1] = foci[point_num - 1].with_intensity(0x00);
    let stm = FocusSTM::from_freq(0.5)
        .with_loop_behavior(LoopBehavior::once())
        .add_foci_from_iter(foci)
        .with_segment(Segment::S1, None);
    autd.send(stm).await?;
    print_msg_and_wait_for_key("何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n2秒後(焦点が再び左端に来た時)に焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること");
    autd.send(SwapSegment::focus_stm(
        Segment::S1,
        TransitionMode::SysTime(DcSysTime::now() + Duration::from_millis(2000)),
    ))
    .await?;
    print_msg_and_wait_for_key("");

    autd.send(SwapSegment::focus_stm(
        Segment::S0,
        TransitionMode::Immidiate,
    ))
    .await?;
    print_msg_and_wait_for_key("再び0.5HzのSTMが適用されたこと");

    print_msg_and_wait_for_key("焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n直ちに焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること");
    autd.send((
        SwapSegment::focus_stm(Segment::S1, TransitionMode::GPIO(GPIOIn::I0)),
        EmulateGPIOIn::new(|_, gpio| gpio == GPIOIn::I0),
    ))
    .await?;

    print_msg_and_wait_for_key("");

    autd.send(Sine::new(150.)).await?;
    let stm = FocusSTM::from_freq(0.5)
        .add_focus(ControlPoint::new(center + Vector3::new(30., 0., 0.)).with_intensity(0xFF))
        .add_focus(ControlPoint::new(center + Vector3::new(0., 30., 0.)).with_intensity(0xFF));
    autd.send(stm).await?;
    let stm = FocusSTM::from_freq(0.5)
        .add_focus(ControlPoint::new(center + Vector3::new(-30., 0., 0.)).with_intensity(0xFF))
        .add_focus(ControlPoint::new(center + Vector3::new(0., -30., 0.)).with_intensity(0xFF))
        .with_segment(Segment::S1, Some(TransitionMode::Ext));
    autd.send(stm).await?;
    print_msg_and_wait_for_key("1秒ごとに焦点が正方形の頂点にジャンプすること");

    {
        autd.send((Static::new(), Null::new())).await?;
        let stm = FocusSTM::from_freq(0.5)
            .add_foci_from_iter(
                (0..2).map(|_| ControlPoint::new(Vector3::zeros()).with_intensity(0x00)),
            )
            .with_loop_behavior(LoopBehavior::once())
            .with_segment(Segment::S1, Some(TransitionMode::SysTime(DcSysTime::now())));
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::MissTransitionTime)),
            autd.send(stm).await
        );
    }

    Ok(())
}

async fn transition_test_gain_stm<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    let center = autd.geometry.center() + Vector3::new(0., 0., 150.0 * MILLIMETER);
    let point_num = 200;
    let radius = 30.0 * MILLIMETER;
    let gen_foci = || {
        (0..point_num).map(|i| {
            let theta = 2.0 * PI * i as f64 / point_num as f64;
            let p = radius * Vector3::new(theta.cos(), theta.sin(), 0.0);
            Focus::new(center + p).with_intensity(0xFF)
        })
    };

    let stm = GainSTM::from_freq(0.5).add_gains_from_iter(gen_foci());
    autd.send(stm).await?;
    print_msg_and_wait_for_key(
        "各デバイスの中心から150mm直上を中心に半径30mmの円周上に0.5HzのSTMが適用されていること",
    );

    let mut foci = gen_foci().rev().collect::<Vec<_>>();
    foci[point_num - 1] = foci[point_num - 1].clone().with_intensity(0x00);
    let stm = GainSTM::from_freq(0.5)
        .with_loop_behavior(LoopBehavior::once())
        .add_gains_from_iter(foci)
        .with_segment(Segment::S1, None);
    autd.send(stm).await?;
    print_msg_and_wait_for_key("何も変化していないこと\n次に, 焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n2秒後(焦点が再び左端に来た時)に焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること");
    autd.send(SwapSegment::gain_stm(
        Segment::S1,
        TransitionMode::SysTime(DcSysTime::now() + Duration::from_millis(2000)),
    ))
    .await?;
    print_msg_and_wait_for_key("");

    autd.send(SwapSegment::gain_stm(
        Segment::S0,
        TransitionMode::Immidiate,
    ))
    .await?;
    print_msg_and_wait_for_key("再び0.5HzのSTMが適用されたこと");

    print_msg_and_wait_for_key("焦点がデバイスの左端に来たときにEnterを押し次のことを確認する\n直ちに焦点軌道が右端にジャンプし逆方向に進み, 1サイクル後に停止すること");
    autd.send((
        SwapSegment::gain_stm(Segment::S1, TransitionMode::GPIO(GPIOIn::I0)),
        EmulateGPIOIn::new(|_, gpio| gpio == GPIOIn::I0),
    ))
    .await?;

    print_msg_and_wait_for_key("");

    autd.send(Sine::new(150.)).await?;
    let stm = GainSTM::from_freq(0.5)
        .add_gain(Focus::new(center + Vector3::new(30., 0., 0.)).with_intensity(0xFF))
        .add_gain(Focus::new(center + Vector3::new(0., 30., 0.)).with_intensity(0xFF));
    autd.send(stm).await?;
    let stm = GainSTM::from_freq(0.5)
        .add_gain(Focus::new(center + Vector3::new(-30., 0., 0.)).with_intensity(0xFF))
        .add_gain(Focus::new(center + Vector3::new(0., -30., 0.)).with_intensity(0xFF))
        .with_segment(Segment::S1, Some(TransitionMode::Ext));
    autd.send(stm).await?;
    print_msg_and_wait_for_key("1秒ごとに焦点が正方形の頂点にジャンプすること");

    {
        autd.send((Static::new(), Null::new())).await?;
        let stm = GainSTM::from_freq(0.5)
            .add_gains_from_iter((0..2).map(|_| Null::new()))
            .with_loop_behavior(LoopBehavior::once())
            .with_segment(Segment::S1, Some(TransitionMode::SysTime(DcSysTime::now())));
        assert_eq!(
            Err(AUTDError::Internal(AUTDInternalError::MissTransitionTime)),
            autd.send(stm).await
        );
    }

    Ok(())
}

pub async fn transition_test<L: Link>(autd: &mut Controller<L>) -> anyhow::Result<()> {
    transition_test_focus_stm(autd).await?;
    transition_test_gain_stm(autd).await?;

    Ok(())
}
