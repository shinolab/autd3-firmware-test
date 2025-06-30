use autd3::{
    core::{
        derive::{Datagram, DeviceFilter},
        link::{Ack, Link, MsgId, RxMessage},
    },
    driver::{
        datagram::Nop,
        firmware::driver::{Driver, OperationHandler},
        firmware::v12_1::{
            V12_1,
            cpu::{check_firmware_err, check_if_msg_is_processed},
            operation::OperationGenerator,
        },
    },
    prelude::*,
};

pub fn err_test<L: Link>(autd: &mut Controller<L, firmware::V12_1>) -> anyhow::Result<()> {
    {
        let nop = Nop;

        let mut g = nop.operation_generator(
            autd.geometry(),
            &autd.environment,
            &DeviceFilter::all_enabled(),
            &V12_1.firmware_limits(),
        )?;
        let mut operations = autd
            .geometry()
            .iter()
            .map(|dev| g.generate(dev))
            .collect::<Vec<_>>();

        autd.link().ensure_is_open()?;

        let err = {
            let mut tx = autd.link_mut().alloc_tx_buffer()?;

            let msg_id = MsgId::new(0x10);
            OperationHandler::pack(msg_id, &mut operations, autd.geometry(), &mut tx, false)?;

            autd.link_mut().send(tx)?;

            let mut rx = vec![RxMessage::new(0x00, Ack::new()); autd.geometry().num_devices()];
            loop {
                autd.link().ensure_is_open()?;
                autd.link_mut().receive(&mut rx)?;

                if check_if_msg_is_processed(msg_id, &rx).all(std::convert::identity) {
                    break;
                }
            }

            rx.iter().try_fold((), |_, r| check_firmware_err(r.ack()))
        };
        assert_eq!(Err(AUTDDriverError::InvalidMessageID), err);
    }

    loop {
        if autd.send(Nop).is_ok() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    autd.send(Clear {})?;

    Ok(())
}
