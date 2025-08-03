use evdev::{
  uinput::{VirtualDevice, VirtualDeviceBuilder}, AbsInfo, AbsoluteAxisType, AttributeSet, BusType, EventType, InputEvent, InputId, Key, PropType, UinputAbsSetup
};
use glam::{DVec2, IVec2};

const RESOLUTION: IVec2 = IVec2::new(1000, 1000);

const DEVICE_VENDOR: u16 = 0x056a; // Wacom Co., Ltd
const DEVICE_PRODUCT: u16 = 0x00e2;
const DEVICE_VERSION: u16 = 0x0100;

pub struct VirtualTablet {
  device: VirtualDevice,
}

impl VirtualTablet {
  pub fn new() -> Self {
    let mut device = VirtualDeviceBuilder::new().unwrap()
      .name("Wacom Co., Ltd VIRTUAL TABLET")
      .input_id(InputId::new(
        BusType::BUS_USB,
        DEVICE_VENDOR,
        DEVICE_PRODUCT,
        DEVICE_VERSION,
      ))
      .with_keys(&AttributeSet::from_iter(&[
        Key::BTN_TOUCH,
        Key::BTN_TOOL_PEN,
        Key::BTN_STYLUS,
        Key::BTN_STYLUS2,
      ])).unwrap()
      .with_properties(&AttributeSet::from_iter(&[
        PropType::DIRECT,
        PropType::POINTER,
      ])).unwrap()
      .with_absolute_axis(
        &UinputAbsSetup::new(
          AbsoluteAxisType::ABS_X,
          AbsInfo::new(0, 0, RESOLUTION.x, 0, 0, RESOLUTION.x)
        )
      ).unwrap()
      .with_absolute_axis(
        &UinputAbsSetup::new(
          AbsoluteAxisType::ABS_Y,
          AbsInfo::new(0, 0, RESOLUTION.y, 0, 0, RESOLUTION.y)
        )
      ).unwrap()
      .with_absolute_axis(
        &UinputAbsSetup::new(
          AbsoluteAxisType::ABS_PRESSURE,
          AbsInfo::new(0, 0, 1, 0, 0, 1)
        )
      ).unwrap()
      .build().unwrap();

    println!(
      "virtual device created, syspath: {}",
      device.get_syspath().unwrap().as_os_str().to_string_lossy()
    );

    Self { device }
  }

  pub fn update(&mut self, new_position: DVec2) {
    // Fast path: direct integer conversion and emission
    let events = [
      InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_X.0, new_position.x as i32),
      InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_Y.0, new_position.y as i32),
      InputEvent::new(EventType::ABSOLUTE, AbsoluteAxisType::ABS_PRESSURE.0, 1),
    ];
    
    // Use expect only in debug builds
    if cfg!(debug_assertions) {
      self.device.emit(&events).expect("Failed to emit events");
    } else {
      let _ = self.device.emit(&events);
    }
  }
}