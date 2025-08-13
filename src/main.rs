/// This code is written by Abhishek in August, 2025.
use rand::Rng; // for the Rng trait (provides random_range)
use raylib::prelude::*;
use std::f32::consts::PI;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 900;
const MAX_BALLS: usize = 200;
const GRAVITY_BASE: f32 = 200.0;

// small Vector2 helpers (explicit, avoids depending on binding method names)
fn v2(x: f32, y: f32) -> Vector2 {
    Vector2::new(x, y)
}
fn add(a: Vector2, b: Vector2) -> Vector2 {
    v2(a.x + b.x, a.y + b.y)
}
fn sub(a: Vector2, b: Vector2) -> Vector2 {
    v2(a.x - b.x, a.y - b.y)
}
fn mul_scalar(v: Vector2, s: f32) -> Vector2 {
    v2(v.x * s, v.y * s)
}
fn length(v: Vector2) -> f32 {
    (v.x * v.x + v.y * v.y).sqrt()
}
fn length_sq(v: Vector2) -> f32 {
    v.x * v.x + v.y * v.y
}
fn normalize(v: Vector2) -> Vector2 {
    let len = length(v);
    if len == 0.0 {
        v2(0.0, 0.0)
    } else {
        mul_scalar(v, 1.0 / len)
    }
}
fn dot(a: Vector2, b: Vector2) -> f32 {
    a.x * b.x + a.y * b.y
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (1.0, 1.0, 1.0),
    };

    Color::new((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8, 255)
}

#[derive(Clone)]
struct Ball {
    pos: Vector2,
    vel: Vector2,
    radius: f32,
    color: Color,
    mass: f32,
    alive: bool,
}

impl Ball {
    fn new(rng: &mut impl Rng) -> Self {
        let radius = rng.random_range(10.0f32..30.0f32);
        Ball {
            pos: v2(
                rng.random_range(radius..(SCREEN_WIDTH as f32 - radius)),
                rng.random_range(radius..(SCREEN_HEIGHT as f32 - radius)),
            ),
            vel: v2(
                rng.random_range(-150.0f32..150.0f32),
                rng.random_range(-150.0f32..150.0f32),
            ),
            radius,
            color: Color::new(
                rng.random_range(50u8..255u8),
                rng.random_range(50u8..255u8),
                rng.random_range(50u8..255u8),
                255,
            ),
            mass: radius * 0.5,
            alive: true,
        }
    }

    fn update(&mut self, dt: f32, gravity: f32, friction: f32) {
        self.vel.y += gravity * dt;
        self.vel.x *= 1.0 - friction * dt;
        self.vel.y *= 1.0 - friction * dt;
        self.pos = add(self.pos, mul_scalar(self.vel, dt));

        if self.pos.x - self.radius < 0.0 {
            self.pos.x = self.radius;
            self.vel.x *= -0.8;
        } else if self.pos.x + self.radius > SCREEN_WIDTH as f32 {
            self.pos.x = SCREEN_WIDTH as f32 - self.radius;
            self.vel.x *= -0.8;
        }

        if self.pos.y - self.radius < 0.0 {
            self.pos.y = self.radius;
            self.vel.y *= -0.8;
        } else if self.pos.y + self.radius > SCREEN_HEIGHT as f32 {
            self.pos.y = SCREEN_HEIGHT as f32 - self.radius;
            self.vel.y *= -0.8;
        }

        let mut h = self.color.r as f32 / 255.0;
        h = (h + 0.2 * dt) % 1.0;
        self.color = hsv_to_rgb(h, 0.8, 0.9);
        self.color.a = 255;
    }

    fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.alive {
            d.draw_circle_v(self.pos, self.radius, self.color);
        }
    }
}

fn balls_collision(b1: &mut Ball, b2: &mut Ball) {
    if !b1.alive || !b2.alive {
        return;
    }
    let delta = sub(b2.pos, b1.pos);
    let dist = length(delta);
    if dist == 0.0 {
        return;
    }
    if dist < b1.radius + b2.radius {
        let penetration = b1.radius + b2.radius - dist;
        let direction = mul_scalar(delta, 1.0 / dist);
        let total_mass = b1.mass + b2.mass;
        b1.pos = sub(
            b1.pos,
            mul_scalar(direction, penetration * (b2.mass / total_mass)),
        );
        b2.pos = add(
            b2.pos,
            mul_scalar(direction, penetration * (b1.mass / total_mass)),
        );

        let rel_vel = sub(b2.vel, b1.vel);
        let vel_along_norm = dot(rel_vel, direction);
        if vel_along_norm > 0.0 {
            return;
        }

        let restitution = 0.9;
        let impulse_mag = -(1.0 + restitution) * vel_along_norm / (1.0 / b1.mass + 1.0 / b2.mass);
        let impulse = mul_scalar(direction, impulse_mag);
        b1.vel = sub(b1.vel, mul_scalar(impulse, 1.0 / b1.mass));
        b2.vel = add(b2.vel, mul_scalar(impulse, 1.0 / b2.mass));
    }
}

#[derive(Clone)]
struct Bullet {
    pos: Vector2,
    vel: Vector2,
    radius: f32,
    alive: bool,
}

impl Bullet {
    fn new(pos: Vector2, dir: Vector2) -> Self {
        Bullet {
            pos,
            vel: mul_scalar(normalize(dir), 500.0),
            radius: 5.0,
            alive: true,
        }
    }
    fn update(&mut self, dt: f32) {
        self.pos = add(self.pos, mul_scalar(self.vel, dt));
        if self.pos.x < 0.0
            || self.pos.x > SCREEN_WIDTH as f32
            || self.pos.y < 0.0
            || self.pos.y > SCREEN_HEIGHT as f32
        {
            self.alive = false;
        }
    }
    fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.alive {
            d.draw_circle_v(self.pos, self.radius, Color::RED);
        }
    }
}

#[derive(Clone)]
struct Particle {
    pos: Vector2,
    vel: Vector2,
    color: Color,
    life: f32,
    radius: f32,
}

impl Particle {
    fn new(pos: Vector2, rng: &mut impl Rng) -> Self {
        let angle = rng.random_range(0.0f32..(2.0 * PI));
        let speed = rng.random_range(100.0f32..300.0f32);
        Particle {
            pos,
            vel: mul_scalar(v2(angle.cos(), angle.sin()), speed),
            color: Color::WHITE,
            life: 1.0,
            radius: rng.random_range(2.0f32..6.0f32),
        }
    }
    fn update(&mut self, dt: f32) {
        self.life -= dt;
        self.pos = add(self.pos, mul_scalar(self.vel, dt));
        self.color.a = ((self.life * 255.0).max(0.0)) as u8;
        self.vel.y += 300.0 * dt;
        self.color.r = (self.color.r as f32 * 0.95) as u8;
        self.color.g = (self.color.g as f32 * 0.95) as u8;
        self.color.b = (self.color.b as f32 * 0.95) as u8;
    }
    fn draw(&self, d: &mut RaylibDrawHandle) {
        if self.life > 0.0 {
            d.draw_circle_v(self.pos, self.radius, self.color);
        }
    }
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Fixed: rand 0.9.2 + raylib 5.5.1")
        .build();

    // <-- Use new rng() API (replaces deprecated thread_rng)
    let mut rng = rand::rng();

    let mut balls: Vec<Ball> = (0..15).map(|_| Ball::new(&mut rng)).collect();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut particles: Vec<Particle> = Vec::new();

    let mut player_pos = v2(SCREEN_WIDTH as f32 / 2.0, SCREEN_HEIGHT as f32 / 2.0);
    let mut player_vel = v2(0.0, 0.0);
    let player_acc = 800.0;
    let player_friction = 10.0;
    let player_radius = 20.0;

    let mut gravity = GRAVITY_BASE;
    let mut friction = 1.5;
    let mut attract_mode = false;
    let mut paused = false;

    while !rl.window_should_close() {
        let dt = rl.get_frame_time();

        if rl.is_key_pressed(KeyboardKey::KEY_P) {
            paused = !paused;
        }

        if !paused {
            // movement
            let mut input_dir = v2(0.0, 0.0);
            if rl.is_key_down(KeyboardKey::KEY_RIGHT) {
                input_dir.x += 1.0;
            }
            if rl.is_key_down(KeyboardKey::KEY_LEFT) {
                input_dir.x -= 1.0;
            }
            if rl.is_key_down(KeyboardKey::KEY_UP) {
                input_dir.y -= 1.0;
            }
            if rl.is_key_down(KeyboardKey::KEY_DOWN) {
                input_dir.y += 1.0;
            }

            if length_sq(input_dir) > 0.0 {
                player_vel = add(
                    player_vel,
                    mul_scalar(normalize(input_dir), player_acc * dt),
                );
            } else {
                player_vel = sub(player_vel, mul_scalar(player_vel, player_friction * dt));
            }

            let max_speed = 400.0;
            if length(player_vel) > max_speed {
                player_vel = mul_scalar(normalize(player_vel), max_speed);
            }

            player_pos = add(player_pos, mul_scalar(player_vel, dt));
            player_pos.x = player_pos
                .x
                .clamp(player_radius, SCREEN_WIDTH as f32 - player_radius);
            player_pos.y = player_pos
                .y
                .clamp(player_radius, SCREEN_HEIGHT as f32 - player_radius);

            // shoot
            if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
                let mouse_pos = rl.get_mouse_position();
                let dir = sub(mouse_pos, player_pos);
                if length(dir) > 0.1 {
                    bullets.push(Bullet::new(player_pos, dir));
                }
            }

            // spawn ball on left click (no distributions import needed)
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) && balls.len() < MAX_BALLS
            {
                let velx = rng.random_range(-200.0f32..200.0f32);
                let vely = rng.random_range(-200.0f32..200.0f32);
                let radius = rng.random_range(10.0f32..30.0f32);
                let b = Ball {
                    pos: rl.get_mouse_position(),
                    vel: v2(velx, vely),
                    radius,
                    color: Color::new(
                        rng.random_range(50u8..255u8),
                        rng.random_range(50u8..255u8),
                        rng.random_range(50u8..255u8),
                        255,
                    ),
                    mass: radius * 0.5,
                    alive: true,
                };
                balls.push(b);
            }

            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_RIGHT) {
                attract_mode = !attract_mode;
            }

            let mouse_pos = rl.get_mouse_position();
            for ball in &mut balls {
                let dir = if attract_mode {
                    sub(mouse_pos, ball.pos)
                } else {
                    sub(ball.pos, mouse_pos)
                };
                let dist_sq = length_sq(dir).max(1.0);
                let force_mag = 100000.0 / dist_sq;
                ball.vel = add(ball.vel, mul_scalar(normalize(dir), force_mag * dt));
                ball.update(dt, gravity, friction);
            }

            // ball collisions
            for i in 0..balls.len() {
                for j in (i + 1)..balls.len() {
                    let (left, right) = balls.split_at_mut(j);
                    balls_collision(&mut left[i], &mut right[0]);
                }
            }

            for b in &mut bullets {
                b.update(dt);
            }

            // bullets vs balls
            for b in &mut bullets {
                if !b.alive {
                    continue;
                }
                for ball in &mut balls {
                    if !ball.alive {
                        continue;
                    }
                    if length(sub(b.pos, ball.pos)) < b.radius + ball.radius {
                        ball.alive = false;
                        b.alive = false;
                        for _ in 0..15 {
                            particles.push(Particle::new(ball.pos, &mut rng));
                        }
                        break;
                    }
                }
            }

            balls.retain(|b| b.alive);
            bullets.retain(|b| b.alive);

            for p in &mut particles {
                p.update(dt);
            }
            particles.retain(|p| p.life > 0.0);

            // tweak physics
            if rl.is_key_down(KeyboardKey::KEY_Q) {
                gravity = (gravity - 50.0 * dt).max(0.0);
            }
            if rl.is_key_down(KeyboardKey::KEY_E) {
                gravity = (gravity + 50.0 * dt).min(1000.0);
            }
            if rl.is_key_down(KeyboardKey::KEY_Z) {
                friction = (friction - 0.5 * dt).max(0.0);
            }
            if rl.is_key_down(KeyboardKey::KEY_C) {
                friction = (friction + 0.5 * dt).min(5.0);
            }
        }

        // Draw (Drawing Logic)
        let mut d: RaylibDrawHandle<'_> = rl.begin_drawing(&thread);
        d.clear_background(Color::RAYWHITE);

        for ball in &balls {
            ball.draw(&mut d);
        }
        for bullet in &bullets {
            bullet.draw(&mut d);
        }
        for p in &particles {
            p.draw(&mut d);
        }

        d.draw_circle_v(player_pos, player_radius, Color::BLACK);
        d.draw_text(
            &format!("Balls: {} (Max {})", balls.len(), MAX_BALLS),
            10,
            10,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Bullets: {}", bullets.len()),
            10,
            40,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Particles: {}", particles.len()),
            10,
            70,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Gravity (Q/E): {:.1}", gravity),
            10,
            100,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!("Friction (Z/C): {:.2}", friction),
            10,
            130,
            20,
            Color::BLACK,
        );
        d.draw_text(
            &format!(
                "Attract mode (Right click): {}",
                if attract_mode { "ON" } else { "OFF" }
            ),
            10,
            160,
            20,
            Color::BLACK,
        );
        d.draw_text(
            "Pause (P) | Shoot (Space) | Add Ball (Left Click) | Move (Arrow Keys)",
            10,
            SCREEN_HEIGHT - 30,
            20,
            Color::DARKGRAY,
        );
        d.draw_fps(10, SCREEN_HEIGHT - 50);
    }
}

// Finally Completed.
// More advanced features would be added gradually.
// will add features one by one to make this great.