mod clear;
mod debug;
mod force_fan;
mod gain;
mod modulation;
mod phase_filter;
mod pulse_width_encoder;
mod silencer;
mod stm_focus;
mod stm_gain;
mod transition;

use colored::*;
use std::io::{self, Write};

use anyhow::Result;

use autd3::{
    derive::*,
    driver::{firmware::version::FirmwareVersion, link::LinkBuilder},
    prelude::*,
};
use autd3_link_soem::{Status, SOEM};

fn print_check(msg: &str) {
    println!("{}: {}", "Check".yellow().bold(), msg);
}

fn print_msg_and_wait_for_key(msg: &str) {
    msg.lines().for_each(|line| {
        print!("{}: ", "Check".yellow().bold());
        println!("{}", line);
    });
    println!("Enterを押して進む...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

async fn run<B: LinkBuilder>(b: B, freq: u32) -> Result<()> {
    let mut autd =
        Controller::builder([AUTD3::new(Vector3::zeros()), AUTD3::new(Vector3::zeros())])
            .with_ultrasound_freq(freq * Hz)
            .open(b)
            .await?;

    autd.send(DebugSettings::new(|_dev, gpio| match gpio {
        GPIOOut::O0 => DebugType::BaseSignal,
        _ => DebugType::None,
    }))
    .await?;
    print_check("各デバイスのGPIO[0]ピンの出力が同期していること");

    let firmware_version = autd.firmware_version().await?;
    assert_eq!(autd.geometry.num_devices(), firmware_version.len());
    firmware_version.iter().for_each(|firm_info| {
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
            firm_info.fpga_version_number_major()
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MINOR,
            firm_info.fpga_version_number_minor()
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
            firm_info.cpu_version_number_major()
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MINOR,
            firm_info.cpu_version_number_minor()
        );
    });

    autd.send(ReadsFPGAState::new(|_| true)).await?;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    autd.fpga_state().await?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    type Test<L> = (
        &'static str,
        fn(
            &'_ mut Controller<L>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<()>> + '_>>,
    );

    let tests: Vec<Test<_>> = vec![
        ("Gainテスト", |autd| Box::pin(gain::gain_test(autd))),
        ("Modulationテスト", |autd| {
            Box::pin(modulation::modulation_test(autd))
        }),
        ("FocusSTMテスト", |autd| {
            Box::pin(stm_focus::stm_focus_test(autd))
        }),
        ("GainSTMテスト", |autd| {
            Box::pin(stm_gain::stm_gain_test(autd))
        }),
        ("Silencerテスト", |autd| {
            Box::pin(silencer::silencer_test(autd))
        }),
        ("PhaseFilterテスト", |autd| {
            Box::pin(phase_filter::phase_filter_test(autd))
        }),
        ("ForceFanテスト", |autd| {
            Box::pin(force_fan::force_fan_test(autd))
        }),
        ("Pulse Width Encoderテスト", |autd| {
            Box::pin(pulse_width_encoder::pwe_test(autd))
        }),
        ("Transitionテスト", |autd| {
            Box::pin(transition::transition_test(autd))
        }),
        ("Debugテスト", |autd| Box::pin(debug::debug_test(autd))),
    ];

    loop {
        tests.iter().enumerate().for_each(|(i, (name, _))| {
            println!("[{}]: {}", i, name);
        });
        println!("[その他]: 終了");
        print!("{}: ", "番号を選択".green().bold());
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        match s.trim().parse::<usize>() {
            Ok(i) if i < tests.len() => {
                (tests[i].1)(&mut autd).await?;
            }
            _ => break,
        }

        autd.send((Null::default(), Silencer::default())).await?;

        clear::clear_test(&mut autd).await?;
    }

    autd.close().await?;

    println!("Ok!");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    print_check("2台の最新ファームウェアを書き込んだデバイスが接続されていること");
    print_check("各デバイスのGPIO[0]ピンとGPIO[1]ピンにオシロスコープを接続していること");
    print_check("各デバイスのGPIOピンに出力がないこと");
    print!("{} (デフォルトは40000): ", "周波数を入力".green().bold());
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    let freq = match s.trim().parse::<u32>() {
        Ok(i) => i,
        _ => 40000,
    };

    let links = vec!["SOEM", "TwinCAT", "Simulator", "Audit"];
    links.iter().enumerate().for_each(|(i, link)| {
        println!("[{}]: {}", i, link);
    });
    print!("{} (デフォルトはSOEM): ", "リンクを選択".green().bold());
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    match s.trim().parse::<usize>() {
        Ok(i) if i < links.len() => match i {
            0 => {
                run(
                    SOEM::builder().with_err_handler(|slave, status| match status {
                        Status::Error => eprintln!("Error [{}]: {}", slave, status),
                        Status::Lost => {
                            eprintln!("Lost [{}]: {}", slave, status);
                            std::process::exit(-1);
                        }
                        Status::StateChanged => {
                            eprintln!("StateChanged [{}]: {}", slave, status)
                        }
                    }),
                    freq,
                )
                .await
            }
            1 => run(autd3_link_twincat::TwinCAT::builder(), freq).await,
            2 => run(autd3_link_simulator::Simulator::builder(8080), freq).await,
            3 => run(autd3::link::Audit::builder(), freq).await,
            _ => unreachable!(),
        },
        _ => {
            run(
                SOEM::builder().with_err_handler(|slave, status| match status {
                    Status::Error => eprintln!("Error [{}]: {}", slave, status),
                    Status::Lost => {
                        eprintln!("Lost [{}]: {}", slave, status);
                        std::process::exit(-1);
                    }
                    Status::StateChanged => eprintln!("StateChanged [{}]: {}", slave, status),
                }),
                freq,
            )
            .await
        }
    }
}
