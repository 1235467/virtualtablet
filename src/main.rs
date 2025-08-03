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

  // Trackpad dimensions from evtest
  const TRACKPAD_MIN_X: f64 = -3678.0;
  const TRACKPAD_MAX_X: f64 = 3934.0;
  const TRACKPAD_MIN_Y: f64 = -2478.0;
  const TRACKPAD_MAX_Y: f64 = 2587.0;
  
  // Calculate the center 1/4 section coordinates
  // Center 1/4: centered both horizontally and vertically, 1/2 width and 1/2 height
  const SECTION_WIDTH: f64 = (TRACKPAD_MAX_X - TRACKPAD_MIN_X) * 0.5;  // 1/2 width
  const SECTION_HEIGHT: f64 = (TRACKPAD_MAX_Y - TRACKPAD_MIN_Y) * 0.5;  // 1/2 height
  
  // Center the section both horizontally and vertically
  const SECTION_MIN_X: f64 = TRACKPAD_MIN_X + ((TRACKPAD_MAX_X - TRACKPAD_MIN_X) - SECTION_WIDTH) * 0.5;
  const SECTION_MAX_X: f64 = SECTION_MIN_X + SECTION_WIDTH;
  const SECTION_MIN_Y: f64 = TRACKPAD_MIN_Y + ((TRACKPAD_MAX_Y - TRACKPAD_MIN_Y) - SECTION_HEIGHT) * 0.5;
  const SECTION_MAX_Y: f64 = SECTION_MIN_Y + SECTION_HEIGHT;
  
  // Calculate scaling factors to map section range to virtual tablet resolution
  const SECTION_RANGE_X: f64 = SECTION_MAX_X - SECTION_MIN_X;
  const SECTION_RANGE_Y: f64 = SECTION_MAX_Y - SECTION_MIN_Y;
  
  // Debug: Print calculated section boundaries
  println!("Trackpad range: X={:.0} to {:.0}, Y={:.0} to {:.0}",
           TRACKPAD_MIN_X, TRACKPAD_MAX_X, TRACKPAD_MIN_Y, TRACKPAD_MAX_Y);
  println!("Center 1/4 section: X={:.0} to {:.0}, Y={:.0} to {:.0}",
           SECTION_MIN_X, SECTION_MAX_X, SECTION_MIN_Y, SECTION_MAX_Y);
  println!("Section dimensions: {:.0} x {:.0}", SECTION_WIDTH, SECTION_HEIGHT);
  
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
          // Map trackpad coordinate to the center 1/4 section
          let normalized_x = (x as f64 - SECTION_MIN_X) / SECTION_RANGE_X;
          cursor_position.x = normalized_x.clamp(0.0, 1.0) * 1000.0;
          updated = true;
        }
        if let Some(y) = pending_y {
          // Map trackpad coordinate to the center 1/4 section
          let normalized_y = (y as f64 - SECTION_MIN_Y) / SECTION_RANGE_Y;
          cursor_position.y = normalized_y.clamp(0.0, 1.0) * 1000.0;
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
