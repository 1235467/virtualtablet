use evdev::{Device, InputEventKind, AbsoluteAxisType, Key};
use glam::DVec2;
use std::collections::HashMap;
use std::time::{Duration, Instant};

mod tablet;
use tablet::VirtualTablet;

mod grab;
use grab::GrabbedDevice;

struct TouchTracker {
  touches: HashMap<i32, DVec2>,
  current_slot: i32,
}

impl TouchTracker {
  fn new() -> Self {
    Self {
      touches: HashMap::new(),
      current_slot: 0,
    }
  }

  fn handle_event(&mut self, event: &evdev::InputEvent) {
    match event.kind() {
      InputEventKind::AbsAxis(axis) => match axis {
        AbsoluteAxisType::ABS_MT_SLOT => self.current_slot = event.value(),
        AbsoluteAxisType::ABS_MT_TRACKING_ID => {
          if event.value() == -1 {
            self.touches.remove(&self.current_slot);
          }
        }
        AbsoluteAxisType::ABS_MT_POSITION_X => {
          self.touches.entry(self.current_slot).or_default().x = event.value() as f64;
        }
        AbsoluteAxisType::ABS_MT_POSITION_Y => {
          self.touches.entry(self.current_slot).or_default().y = event.value() as f64;
        }
        _ => (),
      },
      InputEventKind::Key(key) if key == Key::BTN_TOUCH && event.value() == 0 => {
        self.touches.clear();
      }
      _ => (),
    }
  }

  fn average_position(&self) -> Option<DVec2> {
    if self.touches.is_empty() {
      return None;
    }
    let sum: DVec2 = self.touches.values().copied().sum();
    Some(sum / self.touches.len() as f64)
  }
}

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

  // --- Smart Jump Filter for osu! ---
  // This logic handles two conflicting problems:
  // 1. Sweaty fingers causing single-frame coordinate spikes.
  // 2. osu! gameplay requiring legitimate, large, high-speed jumps.
  // The solution is to require a "confirmation" for any large jump.
  // A jump is only accepted if the *next* input point lands near the jump's destination.

  // If a move is bigger than this, it's a potential jump that needs confirmation.
  const JUMP_DETECTION_THRESHOLD: f64 = 100.0;
  // To confirm a jump, the next point must be this close to the candidate point.
  const JUMP_CONFIRMATION_DISTANCE: f64 = 75.0;

  let mut cursor_position = DVec2::ZERO;
  let mut smoothed_position = DVec2::ZERO;
  // The last cursor position that we're confident is correct.
  let mut last_confirmed_position = DVec2::ZERO;
  let mut last_position = DVec2::ZERO;
  // If a big jump happens, we store it here and wait for the next frame to confirm.
  let mut jump_candidate: Option<DVec2> = None;
  // Handles the very first point we receive.
  let mut has_received_point = false;
  let mut last_update = Instant::now();

  let mut touch_tracker = TouchTracker::new();

  // Smoothing factor for cursor movement to reduce jitter (debounce)
  // Adjust between 0.0 (heavy smoothing) and 1.0 (no smoothing)
  const SMOOTHING_FACTOR: f64 = 0.5;

  // Rate limiting - max 1000 updates per second
  const MIN_UPDATE_INTERVAL: Duration = Duration::from_micros(1000);

  loop {
    // Use a more efficient polling approach
    match device.fetch_events() {
      Ok(events) => {
        for event in events {
          touch_tracker.handle_event(&event);
        }

        if let Some(avg_pos) = touch_tracker.average_position() {
          // Map trackpad coordinate to the center 1/4 section
          let normalized_x = (avg_pos.x - SECTION_MIN_X) / SECTION_RANGE_X;
          cursor_position.x = normalized_x.clamp(0.0, 1.0) * 1000.0;

          let normalized_y = (avg_pos.y - SECTION_MIN_Y) / SECTION_RANGE_Y;
          cursor_position.y = normalized_y.clamp(0.0, 1.0) * 1000.0;

          // The smart jump filter logic starts here.
          if !has_received_point {
            // First point, nothing to compare to. Just accept it.
            last_confirmed_position = cursor_position;
            smoothed_position = cursor_position; // Also prime the smoother.
            has_received_point = true;
          } else {
            if let Some(candidate) = jump_candidate {
              // Last frame we had a potential jump. Does this new point confirm it?
              if cursor_position.distance(candidate) < JUMP_CONFIRMATION_DISTANCE {
                // Looks like a real jump. The cursor followed through. Accept it.
                last_confirmed_position = candidate;
              }
              // If not, it was a fluke. We do nothing and the cursor stays put,
              // effectively ignoring the bad single-frame spike.

              // We've handled the candidate, so clear it for the next frame.
              jump_candidate = None;

            } else {
              // Normal state: no jump candidate pending.
              // Is this new move large enough to be a potential jump?
              if cursor_position.distance(last_confirmed_position) > JUMP_DETECTION_THRESHOLD {
                // It's a big move. Hold on, let's see what the next frame says.
                // Store it as a candidate but don't move the cursor yet.
                jump_candidate = Some(cursor_position);
              } else {
                // Just a regular, small move. Accept it.
                last_confirmed_position = cursor_position;
              }
            }
          }

          // Apply exponential moving average for smoothing to reduce jitter
          // Use the *last confirmed* position as the input for smoothing.
          smoothed_position =
            last_confirmed_position * SMOOTHING_FACTOR + smoothed_position * (1.0 - SMOOTHING_FACTOR);

          let delta = (smoothed_position - last_position).length();
          let now = Instant::now();

          if delta > POSITION_THRESHOLD && now - last_update >= MIN_UPDATE_INTERVAL {
            vtablet.update(smoothed_position);
            last_position = smoothed_position;
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
