#[cfg(any(feature = "serial", feature = "usb"))]
use std::time::Duration;

#[cfg(any(feature = "serial", feature = "usb"))]
use papermint::PapermintError;

#[cfg(feature = "serial")]
use papermint::transport::serial::{DataBits, FlowControl, Parity, SerialTransport, StopBits};

#[cfg(feature = "usb")]
use papermint::transport::usb::UsbTransport;

#[cfg(feature = "serial")]
#[test]
fn test_serial_transport_builder_and_defaults() {
    let transport = SerialTransport::new("/dev/ttyUSB0", 19200);
    assert_eq!(transport.port_path(), "/dev/ttyUSB0");
    assert_eq!(transport.baud_rate(), 19200);
    assert_eq!(transport.flow_control(), FlowControl::None);

    let configured = transport
        .with_baud_rate(115200)
        .flow_control_hardware()
        .with_data_bits(DataBits::Eight)
        .with_stop_bits(StopBits::One)
        .with_parity(Parity::None)
        .with_write_timeout(Duration::from_millis(3500))
        .with_read_timeout(Duration::from_millis(2500));

    assert_eq!(configured.port_path(), "/dev/ttyUSB0");
    assert_eq!(configured.baud_rate(), 115200);
    assert_eq!(configured.flow_control(), FlowControl::Hardware);

    let sw_flow = configured.flow_control_software();
    assert_eq!(sw_flow.flow_control(), FlowControl::Software);

    let no_flow = sw_flow.flow_control_none();
    assert_eq!(no_flow.flow_control(), FlowControl::None);
}

#[cfg(feature = "serial")]
#[tokio::test]
async fn test_serial_transport_connect_nonexistent_port() {
    let mut transport = SerialTransport::new("/dev/nonexistent_pos_printer_99", 9600);
    let result = transport.connect().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        PapermintError::Serial(msg) => {
            assert!(msg.contains("failed to open serial port"));
        }
        other => panic!("expected PapermintError::Serial, got: {other:?}"),
    }
}

#[cfg(feature = "usb")]
#[test]
fn test_usb_transport_builder_and_defaults() {
    let usb = UsbTransport::from_vid_pid(0x04B8, 0x0202)
        .with_serial_number("SN-987654")
        .with_interface(0)
        .with_endpoints(0x01, 0x81)
        .with_write_timeout(Duration::from_millis(4000))
        .with_read_timeout(Duration::from_millis(3000));

    assert_eq!(usb.vendor_id(), Some(0x04B8));
    assert_eq!(usb.product_id(), Some(0x0202));

    let auto = UsbTransport::find_first_printer();
    assert_eq!(auto.vendor_id(), None);
    assert_eq!(auto.product_id(), None);
}

#[cfg(feature = "usb")]
#[tokio::test]
async fn test_usb_transport_connect_nonexistent_device() {
    let mut transport = UsbTransport::from_vid_pid(0xDEAD, 0xBEEF);
    let result = transport.connect().await;
    assert!(result.is_err());
    match result.unwrap_err() {
        PapermintError::DeviceNotFound(msg) => {
            assert!(msg.contains("USB printer not found"));
            assert!(msg.contains("VID: 0xDEAD"));
            assert!(msg.contains("PID: 0xBEEF"));
        }
        PapermintError::Usb(msg) => {
            // In virtualized environments (e.g. WSL/CI without USB sysfs bus)
            assert!(msg.contains("failed to enumerate USB devices") || msg.contains("USB"));
        }
        other => panic!("expected PapermintError::DeviceNotFound or Usb, got: {other:?}"),
    }
}

#[cfg(all(feature = "escpos", feature = "star", feature = "serial"))]
#[test]
fn test_printer_serial_constructors() {
    use papermint::Printer;

    let escpos = Printer::escpos_serial("/dev/ttyUSB0", 19200);
    assert_eq!(escpos.transport().port_path(), "/dev/ttyUSB0");
    assert_eq!(escpos.transport().baud_rate(), 19200);

    let star = Printer::star_serial("COM3", 9600);
    assert_eq!(star.transport().port_path(), "COM3");
    assert_eq!(star.transport().baud_rate(), 9600);
}

#[cfg(all(feature = "escpos", feature = "star", feature = "usb"))]
#[test]
fn test_printer_usb_constructors() {
    use papermint::Printer;

    let escpos = Printer::escpos_usb(0x04B8, 0x0202);
    assert_eq!(escpos.transport().vendor_id(), Some(0x04B8));
    assert_eq!(escpos.transport().product_id(), Some(0x0202));

    let star = Printer::star_usb(0x0519, 0x0001);
    assert_eq!(star.transport().vendor_id(), Some(0x0519));
    assert_eq!(star.transport().product_id(), Some(0x0001));

    let escpos_auto = Printer::escpos_usb_auto();
    assert_eq!(escpos_auto.transport().vendor_id(), None);

    let star_auto = Printer::star_usb_auto();
    assert_eq!(star_auto.transport().vendor_id(), None);
}
