use evdev::{Device, InputEventKind, AbsoluteAxisType};
use glam::DVec2;

mod tablet;
use tablet::VirtualTablet;

mod grab;
use grab::GrabbedDevice;

fn main() {
  let mut device = Device::open("/dev/input/event5")
    .expect("Failed to open evdev device");
  let mut device = GrabbedDevice::grab(&mut device);

  let mut vtablet = VirtualTablet::new();

  let mut cursor_position = DVec2::ZERO;
  loop {
    for event in device.fetch_events().expect("Failed to fetch events") {
      // println!("{:?}", event);
      match event.kind() {
        InputEventKind::AbsAxis(axis) => {
          // TODO: use multitouch events instead
          match axis {
            AbsoluteAxisType::ABS_X => {
              cursor_position.x = event.value() as f64 / 1920.;
              vtablet.update(cursor_position);
            },
            AbsoluteAxisType::ABS_Y => {
              cursor_position.y = event.value() as f64 / 1080.;
              vtablet.update(cursor_position);
            },
            _ => {},
          }
          //println!("{:?}", cursor_position);
        },
        _ => {},
      }
    }
  }
}
