mod clear;
mod debug;
mod gain;
mod modulation;
mod phase_filter;
mod stm_focus;
mod stm_gain;

use colored::*;
use std::io::{self, Write};

use anyhow::Result;

use autd3::{derive::*, prelude::*};
use autd3_link_soem::{Status, SOEM};

fn print_msg_and_wait_for_key(msg: &str) {
    msg.lines().for_each(|line| {
        print!("{}: ", "check: ".yellow().bold());
        println!("{}", line);
    });
    println!("Press Enter to continue...");
    std::io::stdin().read_line(&mut String::new()).unwrap();
}

#[tokio::main]
async fn main() -> Result<()> {
    print_msg_and_wait_for_key(
        "Make sure you have two devices connected that have the latest firmware.\nAlso check that an oscilloscope is connected to GPIO[0] and GPIO[1] pins of each device.\nAnd check if outputs of GPIO pins are low.",
    );

    let mut autd = Controller::builder()
        .add_device(AUTD3::new(Vector3::zeros()))
        .add_device(AUTD3::new(Vector3::zeros()))
        .open(
            SOEM::builder().with_err_handler(|slave, status| match status {
                Status::Error(msg) => eprintln!("Error [{}]: {}", slave, msg),
                Status::Lost(msg) => {
                    eprintln!("Lost [{}]: {}", slave, msg);
                    std::process::exit(-1);
                }
                Status::StateChanged(msg) => eprintln!("StateChanged [{}]: {}", slave, msg),
            }),
        )
        .await?;

    autd.send(ConfigureDebugSettings::new(|_dev| {
        [
            DebugType::BaseSignal,
            DebugType::None,
            DebugType::None,
            DebugType::None,
        ]
    }))
    .await?;
    print_msg_and_wait_for_key("Check if outputs of GPIO[0] are synchronized.");

    let firmware_version = autd.firmware_version().await?;
    assert_eq!(2, firmware_version.len());
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

    autd.send(ConfigureReadsFPGAState::new(|_| true)).await?;

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
        ("Gain test", |autd| Box::pin(gain::gain_test(autd))),
        ("Modulation test", |autd| {
            Box::pin(modulation::modulation_test(autd))
        }),
        ("FocusSTM test", |autd| {
            Box::pin(stm_focus::stm_focus_test(autd))
        }),
        ("GainSTM test", |autd| {
            Box::pin(stm_gain::stm_gain_test(autd))
        }),
        ("PhaseFilter test", |autd| {
            Box::pin(phase_filter::phase_filter_test(autd))
        }),
        ("Debug test", |autd| Box::pin(debug::debug_test(autd))),
    ];

    loop {
        tests.iter().enumerate().for_each(|(i, (name, _))| {
            println!("[{}]: {}", i, name);
        });
        println!("[Others]: Finish");
        print!("{}", "Choose number: ".green().bold());
        io::stdout().flush()?;

        let mut s = String::new();
        io::stdin().read_line(&mut s)?;
        match s.trim().parse::<usize>() {
            Ok(i) if i < tests.len() => {
                (tests[i].1)(&mut autd).await?;
            }
            _ => break,
        }

        println!("press any key to finish...");
        let mut _s = String::new();
        io::stdin().read_line(&mut _s)?;

        autd.send((Null::default(), ConfigureSilencer::default()))
            .await?;

        clear::clear_test(&mut autd).await?;
    }

    autd.close().await?;

    println!("Ok!");
    Ok(())
}
