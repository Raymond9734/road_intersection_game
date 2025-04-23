use rand::prelude::*;
use sdl2::event::Event;
use sdl2::image::{InitFlag, LoadTexture};
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, Instant};
// Constants
const WINDOW_WIDTH: u32 = 900;
const WINDOW_HEIGHT: u32 = 800;
const ROAD_WIDTH: u32 = 70;
const VEHICLE_WIDTH: u32 = 25;
const VEHICLE_HEIGHT: u32 = 35;
const VEHICLE_SPEED: i32 = 2;
const TRAFFIC_LIGHT_SIZE: u32 = 20;
const MIN_VEHICLE_DISTANCE: i32 = 50;
const VEHICLE_SPAWN_COOLDOWN: Duration = Duration::from_millis(1000);
const TRAFFIC_LIGHT_POS_OFFSET: i32 = 20;
const MAX_GREEN_TIME: Duration = Duration::from_secs(4);
const TURN_OFFSET: i32 = 30;
const NUMBER_OF_CARS_FOR_PRIORITY: u32 = 4;

// Directions
#[derive(Debug, Clone, Copy, PartialEq, Hash, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

// Route types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Route {
    Straight,
    Left,
    Right,
}

// Traffic light state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TrafficLightState {
    Red,
    Green,
}
struct TrafficLight {
    position: Point,
    state: TrafficLightState,
    direction: Direction,
    last_change: Instant,
}

struct Vehicle {
    position: Point,
    direction: Direction,
    route: Route,
    // color: Color,
    has_turned: bool,
    has_passed_intersection: bool,
}

struct TrafficSystem<'a> {
    vehicles: Vec<Vehicle>,
    traffic_lights: Vec<TrafficLight>,
    last_spawn_time: Instant,
    vehicle_textures: HashMap<(Direction, Route), Texture<'a>>, // Updated to store Direction and Route
    traffic_light_textures: HashMap<TrafficLightState, Texture<'a>>,
}

impl<'a> TrafficSystem<'a> {
    fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Result<Self, String> {
        let traffic_lights = vec![
            TrafficLight {
                position: Point::new(
                    (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET,
                    WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2,
                ),
                state: TrafficLightState::Red,
                direction: Direction::North,
                last_change: Instant::now(),
            },
            TrafficLight {
                position: Point::new(
                    WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2,
                    (WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET,
                ),
                state: TrafficLightState::Red,
                direction: Direction::South,
                last_change: Instant::now(),
            },
            TrafficLight {
                position: Point::new(
                    (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET,
                    (WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET,
                ),
                state: TrafficLightState::Green,
                direction: Direction::East,
                last_change: Instant::now(),
            },
            TrafficLight {
                position: Point::new(
                    WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2,
                    WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2,
                ),
                state: TrafficLightState::Red,
                direction: Direction::West,
                last_change: Instant::now(),
            },
        ];

        // Load vehicle textures for each Direction and Route combination
        let mut vehicle_textures = HashMap::new();
        let directions = [
            Direction::North,
            Direction::South,
            Direction::East,
            Direction::West,
        ];
        let routes = [Route::Straight, Route::Left, Route::Right];

        for &direction in &directions {
            for &route in &routes {
                let texture_path = match (direction, route) {
                    (Direction::North, Route::Straight) => "assets/vehicles/car_north_straight.png",
                    (Direction::North, Route::Left) => "assets/vehicles/car_north_left.png",
                    (Direction::North, Route::Right) => "assets/vehicles/car_north_right.png",
                    (Direction::South, Route::Straight) => "assets/vehicles/car_south_straight.png",
                    (Direction::South, Route::Left) => "assets/vehicles/car_south_left.png",
                    (Direction::South, Route::Right) => "assets/vehicles/car_south_right.png",
                    (Direction::East, Route::Straight) => "assets/vehicles/car_east_straight.png",
                    (Direction::East, Route::Left) => "assets/vehicles/car_east_left.png",
                    (Direction::East, Route::Right) => "assets/vehicles/car_east_right.png",
                    (Direction::West, Route::Straight) => "assets/vehicles/car_west_straight.png",
                    (Direction::West, Route::Left) => "assets/vehicles/car_west_left.png",
                    (Direction::West, Route::Right) => "assets/vehicles/car_west_right.png",
                };
                vehicle_textures.insert(
                    (direction, route),
                    texture_creator
                        .load_texture(Path::new(texture_path))
                        .map_err(|e| format!("Failed to load {}: {}", texture_path, e))?,
                );
            }
        }

        // Load traffic light textures
        let mut traffic_light_textures = HashMap::new();
        traffic_light_textures.insert(
            TrafficLightState::Red,
            texture_creator
                .load_texture(Path::new("assets/traffic_lights/red.png"))
                .map_err(|e| e.to_string())?,
        );
        traffic_light_textures.insert(
            TrafficLightState::Green,
            texture_creator
                .load_texture(Path::new("assets/traffic_lights/green.png"))
                .map_err(|e| e.to_string())?,
        );

        Ok(TrafficSystem {
            vehicles: Vec::new(),
            traffic_lights,
            last_spawn_time: Instant::now(),
            vehicle_textures,
            traffic_light_textures,
        })
    }
    fn update_traffic_lights(&mut self) {
        // Count waiting vehicles per direction
        let mut vehicle_counts = [
            (Direction::North, 0),
            (Direction::South, 0),
            (Direction::East, 0),
            (Direction::West, 0),
        ];
        for vehicle in &self.vehicles {
            if !vehicle.has_passed_intersection {
                match vehicle.direction {
                    Direction::North => {
                        let stop_y = (WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5;
                        if vehicle.position.y >= stop_y {
                            vehicle_counts[0].1 += 1;
                        }
                    }
                    Direction::South => {
                        let stop_y = WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2;
                        if vehicle.position.y <= stop_y {
                            vehicle_counts[1].1 += 1;
                        }
                    }
                    Direction::East => {
                        let stop_x = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5;
                        if vehicle.position.x <= stop_x {
                            vehicle_counts[2].1 += 1;
                        }
                    }
                    Direction::West => {
                        let stop_x = (WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5;
                        if vehicle.position.x >= stop_x {
                            vehicle_counts[3].1 += 1;
                        }
                    }
                }
            }
        }

        // Sum total vehicles
        let total_vehicles: u32 = vehicle_counts.iter().map(|&(_, count)| count).sum();

        // Find current green light and check conditions
        let mut current_green_idx = None;
        let mut should_change = false;
        let mut priority_direction = None;

        // Check for priority condition: lane with >= 5 cars while another has < 3
        for &(dir, count) in &vehicle_counts {
            if count >= NUMBER_OF_CARS_FOR_PRIORITY {
                // Check if there's a green light in another lane with < 3 cars
                for light in &self.traffic_lights {
                    if light.state == TrafficLightState::Green && light.direction != dir {
                        let current_lane_count = vehicle_counts
                            .iter()
                            .find(|&&(d, _)| d == light.direction)
                            .map(|&(_, count)| count)
                            .unwrap_or(0);
                        if current_lane_count < 3 {
                            priority_direction = Some(dir);
                            should_change = true;
                            break;
                        }
                    }
                }
            }
        }

        // If no priority condition, find lane with most vehicles
        if priority_direction.is_none() {
            let mut max_vehicles = 0;
            for &(dir, count) in &vehicle_counts {
                if count > max_vehicles {
                    max_vehicles = count;
                    priority_direction = Some(dir);
                }
            }

            // Check if current green light should change
            for (i, light) in self.traffic_lights.iter().enumerate() {
                if light.state == TrafficLightState::Green {
                    current_green_idx = Some(i);
                    let elapsed = light.last_change.elapsed();
                    let current_dir = light.direction;
                    let current_lane_vehicles = vehicle_counts
                        .iter()
                        .find(|&&(dir, _)| dir == current_dir)
                        .map(|&(_, count)| count)
                        .unwrap_or(0);

                    // Change if max time reached or no vehicles in current lane
                    should_change = elapsed >= MAX_GREEN_TIME || current_lane_vehicles == 0;
                }
            }
        }

        if total_vehicles == 0 {
            // Set all lights to red if no vehicles
            for light in self.traffic_lights.iter_mut() {
                light.state = TrafficLightState::Red;
                light.last_change = Instant::now();
            }
        } else {
            let next_idx = if let Some(idx) = current_green_idx {
                if should_change {
                    // Choose priority direction or direction with most vehicles
                    let target_dir = priority_direction.unwrap_or(Direction::North);
                    self.traffic_lights
                        .iter()
                        .position(|light| light.direction == target_dir)
                        .unwrap_or((idx + 1) % 4)
                } else {
                    idx // Keep current green if no change needed
                }
            } else {
                // No green light, choose priority direction or direction with vehicles
                let target_dir = priority_direction.unwrap_or(Direction::North);
                self.traffic_lights
                    .iter()
                    .position(|light| light.direction == target_dir)
                    .unwrap_or(0)
            };

            // Update lights
            for (i, light) in self.traffic_lights.iter_mut().enumerate() {
                if i == next_idx {
                    light.state = TrafficLightState::Green;
                    light.last_change = Instant::now();
                } else {
                    light.state = TrafficLightState::Red;
                    light.last_change = Instant::now();
                }
            }
        }
    }
    fn spawn_vehicle(&mut self, direction: Direction) {
        if self.last_spawn_time.elapsed() < VEHICLE_SPAWN_COOLDOWN {
            return;
        }

        let can_spawn = match direction {
            Direction::North => !self.vehicles.iter().any(|v| {
                v.direction == Direction::North
                    && v.position.y
                        > WINDOW_HEIGHT as i32 - VEHICLE_HEIGHT as i32 - MIN_VEHICLE_DISTANCE
            }),
            Direction::South => !self
                .vehicles
                .iter()
                .any(|v| v.direction == Direction::South && v.position.y < MIN_VEHICLE_DISTANCE),
            Direction::East => !self
                .vehicles
                .iter()
                .any(|v| v.direction == Direction::East && v.position.x < MIN_VEHICLE_DISTANCE),
            Direction::West => !self.vehicles.iter().any(|v| {
                v.direction == Direction::West
                    && v.position.x
                        > WINDOW_WIDTH as i32 - VEHICLE_WIDTH as i32 - MIN_VEHICLE_DISTANCE
            }),
        };

        if !can_spawn {
            return;
        }

        let mut rng = rand::thread_rng();
        let options = [Route::Straight, Route::Left, Route::Right];
        let route = options[rng.gen_range(0..3)];

        // let color = match route {
        //     Route::Straight => Color::RGB(0, 0, 255),
        //     Route::Left => Color::RGB(255, 0, 0),
        //     Route::Right => Color::RGB(255, 255, 0),
        // };

        let position = match direction {
            Direction::North => Point::new(
                WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 4 - VEHICLE_WIDTH as i32 / 2,
                WINDOW_HEIGHT as i32,
            ),
            Direction::South => Point::new(
                WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 4 - VEHICLE_WIDTH as i32 / 2,
                0 - VEHICLE_HEIGHT as i32,
            ),
            Direction::East => Point::new(
                0 - VEHICLE_WIDTH as i32,
                WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 4 - VEHICLE_HEIGHT as i32 / 2,
            ),
            Direction::West => Point::new(
                WINDOW_WIDTH as i32,
                WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 4 - VEHICLE_HEIGHT as i32 / 2,
            ),
        };

        let vehicle = Vehicle {
            position,
            direction,
            route,
            // color,
            has_turned: false,
            has_passed_intersection: false,
        };

        self.vehicles.push(vehicle);
        self.last_spawn_time = Instant::now();
    }

    fn spawn_random_vehicle(&mut self) {
        let mut rng = rand::thread_rng();
        let direction = match rng.gen_range(0..4) {
            0 => Direction::North,
            1 => Direction::South,
            2 => Direction::East,
            _ => Direction::West,
        };
        self.spawn_vehicle(direction);
    }

    fn update_vehicles(&mut self) {
        let mut to_remove = Vec::new();
        let vehicle_count = self.vehicles.len();

        let vehicle_positions: Vec<_> = self
            .vehicles
            .iter()
            .map(|v| (v.position, v.direction, v.has_passed_intersection))
            .collect();

        // Define intersection bounds
        let intersection_left = WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2;
        let intersection_right = WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2;
        let intersection_top = WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2;
        let intersection_bottom = WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2;

        let is_in_intersection = |pos: Point| {
            let vehicle_center_x = pos.x + VEHICLE_WIDTH as i32 / 2;
            let vehicle_center_y = pos.y + VEHICLE_HEIGHT as i32 / 2;
            vehicle_center_x > intersection_left
                && vehicle_center_x < intersection_right
                && vehicle_center_y > intersection_top
                && vehicle_center_y < intersection_bottom
        };

        for i in 0..vehicle_count {
            let vehicle = &mut self.vehicles[i];

            if vehicle.position.x < -100
                || vehicle.position.x > WINDOW_WIDTH as i32 + 100
                || vehicle.position.y < -100
                || vehicle.position.y > WINDOW_HEIGHT as i32 + 100
            {
                to_remove.push(i);
                continue;
            }

            let should_stop_for_vehicle = {
                let mut stop = false;
                for (j, (other_pos, other_dir, _)) in vehicle_positions.iter().enumerate() {
                    if i == j {
                        continue;
                    }
                    if vehicle.direction != *other_dir {
                        continue;
                    }
                    match vehicle.direction {
                        Direction::North => {
                            if vehicle.position.x == other_pos.x
                                && vehicle.position.y > other_pos.y
                                && vehicle.position.y - other_pos.y - (VEHICLE_HEIGHT as i32)
                                    < MIN_VEHICLE_DISTANCE
                            {
                                stop = true;
                                break;
                            }
                        }
                        Direction::South => {
                            if vehicle.position.x == other_pos.x
                                && vehicle.position.y < other_pos.y
                                && other_pos.y - vehicle.position.y - (VEHICLE_HEIGHT as i32)
                                    < MIN_VEHICLE_DISTANCE
                            {
                                stop = true;
                                break;
                            }
                        }
                        Direction::East => {
                            if vehicle.position.y == other_pos.y
                                && vehicle.position.x < other_pos.x
                                && other_pos.x - vehicle.position.x - (VEHICLE_WIDTH as i32)
                                    < MIN_VEHICLE_DISTANCE
                            {
                                stop = true;
                                break;
                            }
                        }
                        Direction::West => {
                            if vehicle.position.y == other_pos.y
                                && vehicle.position.x > other_pos.x
                                && vehicle.position.x - other_pos.x - (VEHICLE_WIDTH as i32)
                                    < MIN_VEHICLE_DISTANCE
                            {
                                stop = true;
                                break;
                            }
                        }
                    }
                }
                stop
            };

            let light_state = self
                .traffic_lights
                .iter()
                .find(|light| light.direction == vehicle.direction)
                .map(|light| light.state)
                .unwrap_or(TrafficLightState::Red);

            let at_stop_line = match vehicle.direction {
                Direction::North => {
                    let stop_y = (WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5;
                    vehicle.position.y >= stop_y && vehicle.position.y <= stop_y + 5
                }
                Direction::South => {
                    let stop_y = WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2;
                    vehicle.position.y >= stop_y - VEHICLE_HEIGHT as i32
                        && vehicle.position.y <= stop_y - VEHICLE_HEIGHT as i32 + 5
                }
                Direction::East => {
                    let stop_x = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5;
                    vehicle.position.x >= stop_x - VEHICLE_WIDTH as i32
                        && vehicle.position.x <= stop_x - VEHICLE_WIDTH as i32 + 5
                }
                Direction::West => {
                    let stop_x = (WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5;
                    vehicle.position.x >= stop_x && vehicle.position.x <= stop_x + 5
                }
            };

            let should_stop_at_light =
                if !vehicle.has_passed_intersection && light_state != TrafficLightState::Green {
                    at_stop_line
                } else {
                    false
                };

            let should_wait_for_intersection_clear =
                if at_stop_line && light_state == TrafficLightState::Green {
                    vehicle_positions
                        .iter()
                        .enumerate()
                        .any(|(j, (pos, dir, _))| {
                            j != i && *dir != vehicle.direction && is_in_intersection(*pos)
                        })
                } else {
                    false
                };

            if !should_stop_at_light
                && !should_stop_for_vehicle
                && !should_wait_for_intersection_clear
            {
                let intersection_center_x = WINDOW_WIDTH as i32 / 2;
                let intersection_center_y = WINDOW_HEIGHT as i32 / 2;
                if !vehicle.has_passed_intersection {
                    match vehicle.direction {
                        Direction::North => {
                            if vehicle.route == Route::Right
                                && vehicle.position.y <= intersection_center_y - TURN_OFFSET
                            {
                                vehicle.has_passed_intersection = true;
                            } else if (vehicle.route == Route::Left
                                || vehicle.route == Route::Straight)
                                && vehicle.position.y <= intersection_center_y
                            {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::South => {
                            if vehicle.position.y >= intersection_center_y - TURN_OFFSET {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::East => {
                            if vehicle.route == Route::Right
                                && vehicle.position.x >= intersection_center_x
                            {
                                vehicle.has_passed_intersection = true;
                            } else if (vehicle.route == Route::Left
                                || vehicle.route == Route::Straight)
                                && vehicle.position.x >= intersection_center_x - TURN_OFFSET
                            {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::West => {
                            if vehicle.route == Route::Left
                                && vehicle.position.x <= intersection_center_x
                            {
                                vehicle.has_passed_intersection = true;
                            } else if (vehicle.route == Route::Right
                                || vehicle.route == Route::Straight)
                                && vehicle.position.x <= intersection_center_x - TURN_OFFSET
                            {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                    }
                }

                if vehicle.has_passed_intersection && !vehicle.has_turned {
                    match (vehicle.direction, vehicle.route) {
                        (Direction::North, Route::Left) => {
                            if vehicle.position.y <= intersection_center_y {
                                vehicle.direction = Direction::West;
                                vehicle.position = Point::new(
                                    intersection_center_x
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::North, Route::Right) => {
                            if vehicle.position.y <= intersection_center_y {
                                vehicle.direction = Direction::East;
                                vehicle.position = Point::new(
                                    intersection_center_x
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::South, Route::Left) => {
                            let stop_y = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5;
                            if vehicle.position.y + VEHICLE_WIDTH as i32 >= stop_y - 5 {
                                vehicle.direction = Direction::East;
                                vehicle.position = Point::new(
                                    intersection_center_x + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::South, Route::Right) => {
                            if vehicle.position.y >= intersection_center_y {
                                vehicle.direction = Direction::West;
                                vehicle.position = Point::new(
                                    intersection_center_x + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::East, Route::Left) => {
                            let stop_x = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5;
                            if vehicle.position.x + VEHICLE_WIDTH as i32 >= stop_x - 5 {
                                vehicle.direction = Direction::North;
                                vehicle.position = Point::new(
                                    intersection_center_x
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::East, Route::Right) => {
                            let stop_x = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5;
                            if vehicle.position.x + VEHICLE_WIDTH as i32 >= stop_x - 5 {
                                vehicle.direction = Direction::South;
                                vehicle.position = Point::new(
                                    intersection_center_x + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::West, Route::Left) => {
                            if vehicle.position.x <= intersection_center_x {
                                vehicle.direction = Direction::South;
                                vehicle.position = Point::new(
                                    intersection_center_x + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        (Direction::West, Route::Right) => {
                            if vehicle.position.x <= intersection_center_x {
                                vehicle.direction = Direction::North;
                                vehicle.position = Point::new(
                                    intersection_center_x
                                        - ROAD_WIDTH as i32 / 4
                                        - VEHICLE_WIDTH as i32 / 2,
                                    intersection_center_y + ROAD_WIDTH as i32 / 4
                                        - VEHICLE_HEIGHT as i32 / 2,
                                );
                                vehicle.has_turned = true;
                            }
                        }
                        _ => {}
                    }
                }

                match vehicle.direction {
                    Direction::North => vehicle.position.y -= VEHICLE_SPEED,
                    Direction::South => vehicle.position.y += VEHICLE_SPEED,
                    Direction::East => vehicle.position.x += VEHICLE_SPEED,
                    Direction::West => vehicle.position.x -= VEHICLE_SPEED,
                }
            }
        }

        for i in to_remove.into_iter().rev() {
            self.vehicles.remove(i);
        }
    }

    fn render(&self, canvas: &mut Canvas<Window>) -> Result<(), String> {
        canvas.set_draw_color(Color::RGB(50, 50, 50));
        canvas.clear();

        // Draw roads
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.fill_rect(Rect::new(
            0,
            (WINDOW_HEIGHT as i32 / 2) - (ROAD_WIDTH as i32 / 2),
            WINDOW_WIDTH,
            ROAD_WIDTH,
        ))?;
        canvas.fill_rect(Rect::new(
            (WINDOW_WIDTH as i32 / 2) - (ROAD_WIDTH as i32 / 2),
            0,
            ROAD_WIDTH,
            WINDOW_HEIGHT,
        ))?;

        // Draw road markings
        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let horizontal_center = WINDOW_HEIGHT as i32 / 2;
        for x in (0..WINDOW_WIDTH as i32).step_by(30) {
            canvas.fill_rect(Rect::new(x, horizontal_center, 15, 2))?;
        }
        let vertical_center = WINDOW_WIDTH as i32 / 2;
        for y in (0..WINDOW_HEIGHT as i32).step_by(30) {
            canvas.fill_rect(Rect::new(vertical_center, y, 2, 15))?;
        }

        // Render traffic lights
        for light in &self.traffic_lights {
            let texture = self
                .traffic_light_textures
                .get(&light.state)
                .ok_or("Failed to get traffic light texture")?;
            canvas.copy(
                texture,
                None,
                Rect::new(
                    light.position.x,
                    light.position.y,
                    TRAFFIC_LIGHT_SIZE,
                    TRAFFIC_LIGHT_SIZE,
                ),
            )?;
        }

        // Render vehicles
        for vehicle in &self.vehicles {
            let texture = self
                .vehicle_textures
                .get(&(vehicle.direction, vehicle.route))
                .ok_or("Failed to get vehicle texture")?;

            let (width, height) = match vehicle.direction {
                Direction::North | Direction::South => (VEHICLE_WIDTH, VEHICLE_HEIGHT),
                Direction::East | Direction::West => (VEHICLE_HEIGHT, VEHICLE_WIDTH),
            };
            canvas.copy(
                texture,
                None,
                Rect::new(vehicle.position.x, vehicle.position.y, width, height),
            )?;
        }

        canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let _image_context = sdl2::image::init(InitFlag::PNG)?;

    let window = video_subsystem
        .window(
            "Traffic Intersection Simulation",
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
        )
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    let texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    let mut traffic_system = TrafficSystem::new(&texture_creator)?;
    let mut paused = false;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                } => match keycode {
                    Keycode::Escape => break 'running,
                    Keycode::Up => traffic_system.spawn_vehicle(Direction::North),
                    Keycode::Down => traffic_system.spawn_vehicle(Direction::South),
                    Keycode::Left => traffic_system.spawn_vehicle(Direction::West),
                    Keycode::Right => traffic_system.spawn_vehicle(Direction::East),
                    Keycode::R => traffic_system.spawn_random_vehicle(),
                    Keycode::P => paused = !paused,
                    _ => {}
                },
                _ => {}
            }
        }

        if !paused {
            traffic_system.update_traffic_lights();
            traffic_system.update_vehicles();
        }

        traffic_system.render(&mut canvas)?;
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
