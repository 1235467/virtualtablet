# Virtual Tablet

A tool that reads input from a physical touchpad and maps it to a virtual absolute positioning device, similar to a tablet (pressure is not implemented, just cursor movement currently). And best of all, it works on Wayland!

## Prerequisites

- `cargo`: For building the project.
- `evtest`: A utility to monitor and display kernel input device events.

## Setup and Configuration

The bin file of the project is ``ispen``.

Configuring `ispen` requires editing the source code directly, primarily in [`src/main.rs`](./src/main.rs).

### 1. Find Your Touchpad Device

Run `evtest` without any arguments. It will list all available input devices.

```bash
sudo evtest
```

Select the number corresponding to your touchpad. `evtest` will start printing events when you touch the pad. Note the device path it shows, for example, `/dev/input/event18`. Press `Ctrl+C` to exit.

### 2. Get Touchpad Dimensions

Use `evtest` again, this time with the device path you just found. Pipe the output to `head` to only see the first 30 lines, which contain the device capabilities.

```bash
sudo evtest /dev/input/event18 | head -n 30
```

Look for the `min` and `max` values for `ABS_MT_POSITION_X` and `ABS_MT_POSITION_Y`. They will look something like this:

```
Event code 53 (ABS_MT_POSITION_X)
  Value    213
  Min    -3678
  Max     3934
...
Event code 54 (ABS_MT_POSITION_Y)
  Value   -189
  Min    -2478
  Max     2587
```

### 3. Modify the Source Code

Open [`src/main.rs`](./src/main.rs) and make the following changes:

1.  **Update the device path:**
    In the `main` function, find this line and replace the path with your device's path.

    ```rust
    // Before
    let mut device = Device::open("/dev/input/event18")
    // After
    let mut device = Device::open("/dev/input/event<YOUR_NUMBER>")
    ```

2.  **Update the touchpad dimensions:**
    Find these constants and replace the values with the ones you got from `evtest`.

    ```rust
    // Before
    // Magic Trackpad values
    const TRACKPAD_MIN_X: f64 = -3678.0;
    const TRACKPAD_MAX_X: f64 = 3934.0;
    const TRACKPAD_MIN_Y: f64 = -2478.0;
    const TRACKPAD_MAX_Y: f64 = 2587.0;

    // After (use your values)
    const TRACKPAD_MIN_X: f64 = -3678.0; // Your X min
    const TRACKPAD_MAX_X: f64 = 3934.0;  // Your X max
    const TRACKPAD_MIN_Y: f64 = -2478.0; // Your Y min
    const TRACKPAD_MAX_Y: f64 = 2587.0;  // Your Y max
    ```

### 4. (Optional) Advanced Configuration

You can also tweak other constants to fine-tune performance:

- `SMOOTHING_FACTOR`: Smoothing level, between `0.0` (heavy) and `1.0` (none).
- `JUMP_DETECTION_THRESHOLD`: The minimum distance for a movement to be considered a "jump" that needs confirmation.
- `JUMP_CONFIRMATION_DISTANCE`: The tolerance for confirming a jump.

## Build and Run

After configuration, use `cargo` to build and run the project.

```bash
# Build
cargo build --release

# Run
sudo ./target/release/ispen
```

The program needs to be run with `sudo` to have the necessary permissions to create a virtual device and grab the input device. Once running, your system should detect a new tablet device, and your touchpad input will be mapped to it.