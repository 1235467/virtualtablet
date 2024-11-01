use evdev::Device;

pub struct GrabbedDevice<'a> {
  device: &'a mut Device,
}

impl<'a> GrabbedDevice<'a> {
  pub fn grab(device: &'a mut Device) -> Self{
    device.grab().expect("Failed to grab evdev device");
    println!("device grabbed");
    Self { device }
  }
}

impl<'a> Drop for GrabbedDevice<'a> {
  fn drop(&mut self) {
    self.device.ungrab().expect("Failed to ungrab evdev device");
    println!("device ungrabbed");
  }
}

impl std::ops::Deref for GrabbedDevice<'_> {
  type Target = Device;
  fn deref(&self) -> &Device {
    &self.device
  }
}

impl std::ops::DerefMut for GrabbedDevice<'_> {
  fn deref_mut(&mut self) -> &mut Device {
    &mut self.device
  }
}