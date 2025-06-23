mod clear;
mod debug;
mod err;
mod force_fan;
mod gain;
mod modulation;
mod phase_corr;
mod pulse_width_encoder;
mod silencer;
mod stm_focus;
mod stm_gain;
mod transition;

use colored::*;
use std::io::{self, Write};

use anyhow::Result;

use autd3::{core::link::Link, driver::firmware::version::FirmwareVersion, prelude::*};
// use autd3_link_soem::{SOEM, Status};

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

fn run<L: Link>(link: L) -> Result<()> {
    let mut autd =
        Controller::<_, firmware::Latest>::open_with([AUTD3::default(), AUTD3::default()], link)?;

    autd.send(GPIOOutputs::new(|_dev, gpio| match gpio {
        GPIOOut::O0 => Some(GPIOOutputType::BaseSignal),
        _ => None,
    }))?;
    print_check("各デバイスのGPIO[0]ピンの出力が同期していること");

    let firmware_version = autd.firmware_version()?;
    assert_eq!(autd.geometry().num_devices(), firmware_version.len());
    firmware_version.iter().for_each(|firm_info| {
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
            firm_info.fpga.major
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MINOR,
            firm_info.fpga.minor
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MAJOR,
            firm_info.cpu.major
        );
        assert_eq!(
            FirmwareVersion::LATEST_VERSION_NUM_MINOR,
            firm_info.cpu.minor
        );
    });

    autd.send(ReadsFPGAState::new(|_| true))?;
    std::thread::sleep(std::time::Duration::from_millis(200));
    autd.fpga_state()?.iter().for_each(|state| {
        assert!(state.is_some());
        let state = state.unwrap();
        assert_eq!(Segment::S0, state.current_mod_segment());
        assert_eq!(Some(Segment::S0), state.current_gain_segment());
        assert_eq!(None, state.current_stm_segment());
    });

    type Test<L> = (
        &'static str,
        fn(&'_ mut Controller<L, firmware::Latest>) -> anyhow::Result<()>,
    );

    let tests: Vec<Test<_>> = vec![
        ("Gainテスト", |autd| gain::gain_test(autd)),
        ("Modulationテスト", |autd| {
            modulation::modulation_test(autd)
        }),
        ("FociSTMテスト", |autd| stm_focus::stm_focus_test(autd)),
        ("GainSTMテスト", |autd| stm_gain::stm_gain_test(autd)),
        ("Silencerテスト", |autd| silencer::silencer_test(autd)),
        ("ForceFanテスト", |autd| force_fan::force_fan_test(autd)),
        ("Pulse Width Encoderテスト", |autd| {
            pulse_width_encoder::pwe_test(autd)
        }),
        ("Phase Correctionテスト", |autd| {
            phase_corr::phase_corr_test(autd)
        }),
        ("Transitionテスト", |autd| {
            transition::transition_test(autd)
        }),
        ("Debugテスト", |autd| debug::debug_test(autd)),
        ("Errorテスト", |autd| err::err_test(autd)),
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
                (tests[i].1)(&mut autd)?;
            }
            _ => break,
        }

        autd.send((Null::default(), Silencer::default()))?;

        clear::clear_test(&mut autd)?;
    }

    autd.close()?;

    println!("Ok!");
    Ok(())
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    print_check("2台の最新ファームウェアを書き込んだデバイスが接続されていること");
    print_check("各デバイスのGPIO[0]ピンとGPIO[1]ピンにオシロスコープを接続していること");
    print_check("各デバイスのGPIOピンに出力がないこと");

    let links = vec!["SOEM", "TwinCAT", "Simulator", "Audit"];
    links.iter().enumerate().for_each(|(i, link)| {
        println!("[{}]: {}", i, link);
    });
    print!("{} (デフォルトはSOEM): ", "リンクを選択".green().bold());
    io::stdout().flush()?;
    let mut s = String::new();
    io::stdin().read_line(&mut s)?;
    match s.trim().parse::<usize>().unwrap_or(0) {
        1 => run(autd3_link_twincat::TwinCAT::new()?),
        2 => run(autd3_link_simulator::Simulator::new(
            "127.0.0.1:8080".parse()?,
        )),
        3 => run(autd3::link::Audit::latest(
            autd3::link::AuditOption::default(),
        )),
        _ => run(autd3_link_twincat::TwinCAT::new()?),
        // _ => run(SOEM::new(
        //     |slave, status| {
        //         eprintln!("slave[{}]: {}", slave, status);
        //         if status == Status::Lost {
        //             std::process::exit(-1);
        //         }
        //     },
        //     Default::default(),
        // )),
    }
}
