# Traffic Intersection Simulation

## Overview

The **Traffic Intersection Simulation** is a 2D top-down simulation of a four-way road intersection, built using **Rust** and the **SDL2** library. The project models realistic traffic behavior, including vehicles navigating through an intersection with traffic lights controlling the flow. Vehicles approach from four directions (North, South, East, West) and can follow one of three routes: Straight, Left, or Right. Each route is visually distinguished by unique vehicle textures, enhancing the simulation's realism.

The traffic lights operate dynamically, prioritizing directions with higher vehicle counts or switching based on a timer. The simulation includes collision avoidance, ensuring vehicles maintain a safe distance, and supports interactive controls for spawning vehicles and pausing the simulation.

This project is ideal for learning about traffic management algorithms, game development with Rust, and SDL2 for rendering graphics. It serves as a foundation for more complex traffic simulations or educational tools.

## Features

- **Realistic Traffic Flow**:
  - Vehicles move in four directions (North, South, East, West) with three route options (Straight, Left, Right).
  - Vehicles stop at red lights, wait for clear intersections, and maintain safe distances from each other.
- **Dynamic Traffic Lights**:
  - Traffic lights switch between Red and Green states based on vehicle counts and a maximum green time (4 seconds).
  - Prioritizes directions with 4 or more vehicles when another direction has fewer than 3 vehicles.
  - All lights turn red when no vehicles are present.
- **Custom Vehicle Textures**:
  - Each direction and route combination (e.g., North-Left, South-Right) uses a unique PNG texture.
  - Textures are color-coded (e.g., blue for Straight, red for Left, yellow for Right) for visual distinction.
- **Interactive Controls**:
  - Spawn vehicles manually in specific directions or randomly.
  - Pause and resume the simulation.
  - Exit the simulation with the Escape key.
- **Smooth Rendering**:
  - Runs at 60 FPS with SDL2 for efficient rendering.
  - Roads, dashed lane markings, traffic lights, and vehicles are rendered with proper scaling and orientation.
- **Extensible Design**:
  - Modular code structure allows easy addition of features like yellow lights, pedestrian crossings, or advanced traffic algorithms.

## Requirements

To build and run the Traffic Intersection Simulation, you need the following:

### Software
- **Rust**: Version 1.56 or later (stable recommended).
  - Install via [rustup](https://rustup.rs/): `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Cargo**: Included with Rust for dependency management.
- **SDL2 Development Libraries**:
  - Required for rendering and handling graphics.
  - Install instructions vary by operating system (see below).
- **Git**: For cloning the repository (optional if downloading manually).
  - Install via your package manager (e.g., `sudo apt install git` on Ubuntu).

### Operating System
- **Linux** (e.g., Ubuntu, Fedora)
- **macOS**
- **Windows** (via MSYS2 or WSL recommended for easier SDL2 setup)

### SDL2 Installation
SDL2 and its image extension (`SDL2_image`) are required for rendering PNG textures. Install them as follows:

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libsdl2-dev libsdl2-image-dev
```

#### macOS (using Homebrew)
```bash
brew install sdl2 sdl2_image
```

#### Windows (using MSYS2)
1. Install [MSYS2](https://www.msys2.org/).
2. Open the MSYS2 MinGW 64-bit terminal and run:
```bash
pacman -S mingw-w64-x86_64-SDL2 mingw-w64-x86_64-SDL2_image
```
3. Ensure the MinGW bin directory (e.g., `C:\msys64\mingw64\bin`) is in your system PATH.

For other setups, refer to the [rust-sdl2 README](https://github.com/Rust-SDL2/rust-sdl2#requirements).

### Assets
The project requires PNG textures for vehicles and traffic lights, located in the `assets/` directory. Ensure the following structure:
```
project/
├── assets/
│   ├── vehicles/
│   │   ├── car_north_straight.png
│   │   ├── car_north_left.png
│   │   ├── car_north_right.png
│   │   ├── car_south_straight.png
│   │   ├── car_south_left.png
│   │   ├── car_south_right.png
│   │   ├── car_east_straight.png
│   │   ├── car_east_left.png
│   │   ├── car_east_right.png
│   │   ├── car_west_straight.png
│   │   ├── car_west_left.png
│   │   ├── car_west_right.png
│   ├── traffic_lights/
│   │   ├── red.png
│   │   ├── green.png
```
- **Vehicle Textures**: 25x35 pixels (width x height), PNG format with transparent backgrounds. Each texture corresponds to a direction (North, South, East, West) and route (Straight, Left, Right). Example:
  - `car_north_straight.png`: Blue car facing up.
  - `car_north_left.png`: Red car facing up.
  - `car_north_right.png`: Yellow car facing up.
- **Traffic Light Textures**: 20x20 pixels, PNG format with transparent backgrounds. `red.png` and `green.png` represent Red and Green states.
- **Note**: You must provide these assets or source them from platforms like [OpenGameArt.org](https://opengameart.org/) or [Flaticon](https://www.flaticon.com/). Ensure compliance with asset licenses (e.g., Creative Commons).

## Installation

1. **Clone or Download the Repository**:
   ```bash
   git clone <repository-url>
   cd traffic-intersection-simulation
   ```
   Alternatively, download the source code as a ZIP and extract it.

2. **Set Up Assets**:
   - Create the `assets/vehicles/` and `assets/traffic_lights/` directories in the project root.
   - Place the required PNG files (listed above) in their respective directories.
   - Verify file names match exactly (e.g., `car_north_straight.png`).

3. **Install Dependencies**:
   - Ensure Rust and SDL2 are installed (see Requirements).
   - Cargo will automatically fetch Rust dependencies when building.

4. **Verify Project Structure**:
   ```
   traffic-intersection-simulation/
   ├── assets/
   │   ├── vehicles/
   │   │   ├── car_north_straight.png
   │   │   ├── ...
   │   ├── traffic_lights/
   │   │   ├── red.png
   │   │   ├── green.png
   ├── src/
   │   ├── main.rs
   ├── Cargo.toml
   ├── README.md
   ```

## Building and Running

1. **Build the Project**:
   Navigate to the project directory and run:
   ```bash
   cargo build --release
   ```
   - The `--release` flag optimizes the binary for performance.
   - This fetches dependencies (e.g., `sdl2`, `rand`) and compiles the code.

2. **Run the Simulation**:
   ```bash
   cargo run --release
   ```
   - Launches a 900x800 window displaying the intersection.
   - If you encounter errors (e.g., “Failed to load assets/vehicles/car_north_straight.png”), ensure the asset files exist and paths are correct.

3. **Controls**:
   - **Up Arrow**: Spawn a vehicle from the South (moving North).
   - **Down Arrow**: Spawn a vehicle from the North (moving South).
   - **Left Arrow**: Spawn a vehicle from the East (moving West).
   - **Right Arrow**: Spawn a vehicle from the West (moving East).
   - **R**: Spawn a vehicle in a random direction.
   - **P**: Pause or resume the simulation.
   - **Escape**: Exit the simulation.

4. **Troubleshooting**:
   - **SDL2 Errors**: Ensure SDL2 and SDL2_image libraries are installed and accessible. Check library paths (e.g., `LD_LIBRARY_PATH` on Linux).
   - **Asset Errors**: Verify PNG files are in `assets/vehicles/` and `assets/traffic_lights/`. File names are case-sensitive.
   - **Performance Issues**: Ensure you’re using the `--release` flag for optimized performance. Reduce vehicle spawn rate by increasing `VEHICLE_SPAWN_COOLDOWN` if needed.

## Project Details

### Code Structure
- **Source File**: `src/main.rs`
- **Key Components**:
  - **Enums**:
    - `Direction`: North, South, East, West.
    - `Route`: Straight, Left, Right.
    - `TrafficLightState`: Red, Green.
  - **Structs**:
    - `TrafficLight`: Manages position, state, direction, and timing.
    - `Vehicle`: Tracks position, direction, route, and intersection status.
    - `TrafficSystem`: Core simulation logic, including vehicle and traffic light management, texture loading, and rendering.
  - **Methods**:
    - `TrafficSystem::new`: Initializes the simulation, loading textures and setting up traffic lights.
    - `update_traffic_lights`: Updates light states based on vehicle counts and timers.
    - `update_vehicles`: Moves vehicles, handles turns, and removes off-screen vehicles.
    - `render`: Draws roads, lane markings, traffic lights, and vehicles.
    - `spawn_vehicle`: Adds vehicles with random routes, respecting spawn cooldowns and distance checks.

### Simulation Logic
- **Intersection Layout**:
  - A crossroad with two roads (70 pixels wide) intersecting at the center of a 900x800 window.
  - Traffic lights are positioned at each approach (North, South, East, West).
- **Vehicle Behavior**:
  - Vehicles spawn at the edges and move toward the intersection.
  - Stop at red lights or if another vehicle is too close (50-pixel minimum distance).
  - Wait for a clear intersection before proceeding on green.
  - Turn Left or Right at the intersection based on their route, adjusting direction and position.
- **Traffic Light Logic**:
  - One direction has a green light at a time, others are red.
  - Switches to prioritize directions with 4+ vehicles if another has <3 vehicles.
  - Maximum green time is 4 seconds, or switches if no vehicles are waiting.
  - All lights turn red when no vehicles are present.
- **Rendering**:
  - Roads are black rectangles with white dashed lane markings.
  - Traffic lights use 20x20 PNGs (red/green).
  - Vehicles use 25x35 PNGs, scaled and oriented based on direction (North/South: upright, East/West: rotated).

### Dependencies
Defined in `Cargo.toml`:
```toml
[package]
name = "road_intersection"
version = "0.1.0"
edition = "2024"

[dependencies]
sdl2 = { version = "0.37", features = ["image"] }
rand = "0.8.4"
```
- **sdl2**: Handles window creation, rendering, and PNG texture loading.
- **rand**: Generates random directions and routes for vehicles.

## Customization

To extend or modify the simulation, consider:
- **Adding Yellow Lights**: Update `TrafficLightState` to include Yellow, add `yellow.png`, and implement transition logic in `update_traffic_lights`.
- **Adjusting Parameters**: Modify constants in `src/main.rs`:
  - `VEHICLE_SPEED`: Change vehicle movement speed (default: 2 pixels/frame).
  - `VEHICLE_SPAWN_COOLDOWN`: Adjust spawn frequency (default: 1000ms).
  - `MAX_GREEN_TIME`: Change maximum green light duration (default: 4s).
  - `NUMBER_OF_CARS_FOR_PRIORITY`: Adjust priority threshold (default: 4 cars).
- **New Textures**: Replace PNGs in `assets/` with custom sprites, ensuring correct dimensions and transparency.
- **Debug Logging**: Add `println!` statements in `render` or `update_vehicles` to track vehicle states or texture usage.

## Known Issues
- **Texture Orientation**: Ensure vehicle textures (e.g., `car_north_right.png`) face the correct direction (e.g., up for North). Incorrectly oriented sprites may require manual rotation or code-based rotation using `canvas.copy_ex`.
- **Asset Dependency**: The project requires user-provided PNGs. Missing or misnamed files cause runtime errors.
- **Simplified Turns**: Vehicles teleport to new positions during turns, which may look abrupt. Future versions could implement smooth turning animations.

## Contributing
Contributions are welcome! To contribute:
1. Fork the repository.
2. Create a feature branch (`git checkout -b feature/YourFeature`).
3. Commit changes (`git commit -m "Add YourFeature"`).
4. Push to the branch (`git push origin feature/YourFeature`).
5. Open a pull request.

Please include tests and update this README for new features.

## License
This project is licensed under the MIT License. See `LICENSE` file for details. Note that assets (PNG files) may have separate licenses (e.g., Creative Commons). Ensure compliance with asset licenses when redistributing.

