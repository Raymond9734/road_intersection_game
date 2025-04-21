use rand::prelude::*;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::{Duration, Instant};

// Constants
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const ROAD_WIDTH: u32 = 90;
const VEHICLE_WIDTH: u32 = 15;
const VEHICLE_HEIGHT: u32 = 20;
const VEHICLE_SPEED: i32 = 2;
const TRAFFIC_LIGHT_SIZE: u32 = 20;
const MIN_VEHICLE_DISTANCE: i32 = 30;
const VEHICLE_SPAWN_COOLDOWN: Duration = Duration::from_millis(1000);
const TRAFFIC_LIGHT_CYCLE: Duration = Duration::from_secs(10);
const TRAFFIC_LIGHT_POS_OFFSET: i32 = 20;

// Directions
#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    North,
    South,
    East,
    West,
}

// Route types
#[derive(Debug, Clone, Copy, PartialEq)]
enum Route {
    Straight,
    Left,
    Right,
}

// Traffic light state
#[derive(Debug, Clone, Copy, PartialEq)]
enum TrafficLightState {
    Red,
    Green,
}

struct TrafficLight {
    position: Point,
    state: TrafficLightState,
    direction: Direction,
}

struct Vehicle {
    position: Point,
    direction: Direction,
    route: Route,
    color: Color,
    has_turned: bool,
    has_passed_intersection: bool,
}

struct TrafficSystem {
    vehicles: Vec<Vehicle>,
    traffic_lights: Vec<TrafficLight>,
    last_spawn_time: Instant,
    traffic_light_switch_time: Instant,
    current_ns_light_state: TrafficLightState,
    current_ew_light_state: TrafficLightState,
}

impl TrafficSystem {
    fn new() -> Self {
        let traffic_lights = vec![
            // North (bottom-left corner, controls North-bound vehicles)
            TrafficLight {
                position: Point::new(
                    (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET, // x=335
                    WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2, // y=345
                ),
                state: TrafficLightState::Red,
                direction: Direction::North,
            },
            // South (top-right corner, controls South-bound vehicles)
            TrafficLight {
                position: Point::new(
                    WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2, // x=445
                    (WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET, // y=235
                ),
                state: TrafficLightState::Red,
                direction: Direction::South,
            },
            // East (top-left corner, controls East-bound vehicles)
            TrafficLight {
                position: Point::new(
                    (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET, // x=335
                    (WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2) - TRAFFIC_LIGHT_POS_OFFSET, // y=235
                ),
                state: TrafficLightState::Green,
                direction: Direction::East,
            },
            // West (bottom-right corner, controls West-bound vehicles)
            TrafficLight {
                position: Point::new(
                    WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2, // x=445
                    WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2, // y=345
                ),
                state: TrafficLightState::Green,
                direction: Direction::West,
            },
        ];

        TrafficSystem {
            vehicles: Vec::new(),
            traffic_lights,
            last_spawn_time: Instant::now(),
            traffic_light_switch_time: Instant::now(),
            current_ns_light_state: TrafficLightState::Red,
            current_ew_light_state: TrafficLightState::Green,
        }
    }

    fn update_traffic_lights(&mut self) {
        if self.traffic_light_switch_time.elapsed() >= TRAFFIC_LIGHT_CYCLE {
            self.current_ns_light_state = match self.current_ns_light_state {
                TrafficLightState::Red => TrafficLightState::Green,
                TrafficLightState::Green => TrafficLightState::Red,
            };
            self.current_ew_light_state = match self.current_ew_light_state {
                TrafficLightState::Red => TrafficLightState::Green,
                TrafficLightState::Green => TrafficLightState::Red,
            };

            for light in &mut self.traffic_lights {
                match light.direction {
                    Direction::North | Direction::South => {
                        light.state = self.current_ns_light_state
                    }
                    Direction::East | Direction::West => light.state = self.current_ew_light_state,
                }
            }

            self.traffic_light_switch_time = Instant::now();
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

        let color = match route {
            Route::Straight => Color::RGB(0, 0, 255), // Blue
            Route::Left => Color::RGB(255, 0, 0),     // Red
            Route::Right => Color::RGB(255, 255, 0),  // Yellow
        };

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
            color,
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

            let should_stop_at_light = {
                if vehicle.has_passed_intersection {
                    false
                } else {
                    match vehicle.direction {
                        Direction::North => {
                            let stop_y = (WINDOW_HEIGHT as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5; // y=345
                            vehicle.position.y >= stop_y
                                && vehicle.position.y <= stop_y + 5
                                && self.current_ns_light_state == TrafficLightState::Red
                        }
                        Direction::South => {
                            let stop_y = WINDOW_HEIGHT as i32 / 2 - ROAD_WIDTH as i32 / 2; // y=255
                            vehicle.position.y >= stop_y - VEHICLE_HEIGHT as i32
                                && vehicle.position.y <= stop_y - VEHICLE_HEIGHT as i32 + 5
                                && self.current_ns_light_state == TrafficLightState::Red
                        }
                        Direction::East => {
                            let stop_x = (WINDOW_WIDTH as i32 / 2 - ROAD_WIDTH as i32 / 2) - 5; // x=355
                            vehicle.position.x >= stop_x - VEHICLE_WIDTH as i32
                                && vehicle.position.x <= stop_x - VEHICLE_WIDTH as i32 + 5
                                && self.current_ew_light_state == TrafficLightState::Red
                        }
                        Direction::West => {
                            let stop_x = (WINDOW_WIDTH as i32 / 2 + ROAD_WIDTH as i32 / 2) - 5; // x=445
                            vehicle.position.x >= stop_x
                                && vehicle.position.x <= stop_x + 5
                                && self.current_ew_light_state == TrafficLightState::Red
                        }
                    }
                }
            };

            if !should_stop_at_light && !should_stop_for_vehicle {
                let intersection_center_x = WINDOW_WIDTH as i32 / 2;
                let intersection_center_y = WINDOW_HEIGHT as i32 / 2;

                if !vehicle.has_passed_intersection {
                    match vehicle.direction {
                        Direction::North => {
                            if vehicle.position.y <= intersection_center_y {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::South => {
                            if vehicle.position.y >= intersection_center_y {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::East => {
                            if vehicle.position.x >= intersection_center_x {
                                vehicle.has_passed_intersection = true;
                            }
                        }
                        Direction::West => {
                            if vehicle.position.x <= intersection_center_x {
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
                            if vehicle.position.y >= intersection_center_y {
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
                            if vehicle.position.x >= intersection_center_x {
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
                            if vehicle.position.x >= intersection_center_x {
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

        canvas.set_draw_color(Color::RGB(255, 255, 255));
        let horizontal_center = WINDOW_HEIGHT as i32 / 2;
        for x in (0..WINDOW_WIDTH as i32).step_by(30) {
            canvas.fill_rect(Rect::new(x, horizontal_center, 15, 2))?;
        }
        let vertical_center = WINDOW_WIDTH as i32 / 2;
        for y in (0..WINDOW_HEIGHT as i32).step_by(30) {
            canvas.fill_rect(Rect::new(vertical_center, y, 2, 15))?;
        }

        for light in &self.traffic_lights {
            let color = match light.state {
                TrafficLightState::Red => Color::RGB(255, 0, 0),
                TrafficLightState::Green => Color::RGB(0, 255, 0),
            };
            canvas.set_draw_color(color);
            canvas.fill_rect(Rect::new(
                light.position.x,
                light.position.y,
                TRAFFIC_LIGHT_SIZE,
                TRAFFIC_LIGHT_SIZE,
            ))?;
        }

        for vehicle in &self.vehicles {
            canvas.set_draw_color(vehicle.color);
            let (width, height) = match vehicle.direction {
                Direction::North | Direction::South => (VEHICLE_WIDTH, VEHICLE_HEIGHT),
                Direction::East | Direction::West => (VEHICLE_HEIGHT, VEHICLE_WIDTH),
            };
            canvas.fill_rect(Rect::new(
                vehicle.position.x,
                vehicle.position.y,
                width,
                height,
            ))?;
        }

        canvas.present();
        Ok(())
    }
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

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
    let mut event_pump = sdl_context.event_pump()?;

    let mut traffic_system = TrafficSystem::new();
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
                    Keycode::Up => traffic_system.spawn_vehicle(Direction::South),
                    Keycode::Down => traffic_system.spawn_vehicle(Direction::North),
                    Keycode::Left => traffic_system.spawn_vehicle(Direction::East),
                    Keycode::Right => traffic_system.spawn_vehicle(Direction::West),
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
