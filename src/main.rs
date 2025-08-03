use evdev::{Device, InputEventKind, AbsoluteAxisType};
use glam::DVec2;
use std::time::{Duration, Instant};

mod tablet;
use tablet::VirtualTablet;

mod grab;
use grab::GrabbedDevice;

fn main() {
  let mut device = Device::open("/dev/input/event18")
    .expect("Failed to open evdev device");
  let mut device = GrabbedDevice::grab(&mut device);

  let mut vtablet = VirtualTablet::new();

  // Pre-calculate scaling factors
  const SCALE_X: f64 = 1000.0 / 1920.0;
  const SCALE_Y: f64 = 1000.0 / 1080.0;
  
  // Threshold for position updates (reduces unnecessary updates)
  const POSITION_THRESHOLD: f64 = 1.0;
  
  let mut cursor_position = DVec2::ZERO;
  let mut last_position = DVec2::ZERO;
  let mut last_update = Instant::now();
  
  // Rate limiting - max 1000 updates per second
  const MIN_UPDATE_INTERVAL: Duration = Duration::from_micros(1000);
  
  loop {
    // Use a more efficient polling approach
    match device.fetch_events() {
      Ok(events) => {
        let mut pending_x = None;
        let mut pending_y = None;
        
        // Process all events in batch, only keeping the latest values
        for event in events {
          if let InputEventKind::AbsAxis(axis) = event.kind() {
            match axis {
              AbsoluteAxisType::ABS_X => pending_x = Some(event.value()),
              AbsoluteAxisType::ABS_Y => pending_y = Some(event.value()),
              _ => {},
            }
          }
        }
        
        // Apply updates only if we have new data
        let mut updated = false;
        if let Some(x) = pending_x {
          cursor_position.x = x as f64 * SCALE_X;
          updated = true;
        }
        if let Some(y) = pending_y {
          cursor_position.y = y as f64 * SCALE_Y;
          updated = true;
        }
        
        // Batch update with rate limiting
        if updated {
          let delta = (cursor_position - last_position).length();
          let now = Instant::now();
          
          if delta > POSITION_THRESHOLD && now - last_update >= MIN_UPDATE_INTERVAL {
            vtablet.update(cursor_position);
            last_position = cursor_position;
            last_update = now;
          }
        }
      }
      Err(_) => {
        // On error, sleep a bit longer to avoid busy waiting
        std::thread::sleep(Duration::from_millis(1));
        continue;
      }
    }
    
    // Adaptive sleep based on update frequency
    let time_since_update = last_update.elapsed();
    if time_since_update > Duration::from_millis(10) {
      std::thread::sleep(Duration::from_micros(500));
    } else {
      std::thread::sleep(Duration::from_micros(50));
    }
  }
}
