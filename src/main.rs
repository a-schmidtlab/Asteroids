use ggez::{
    event,
    graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Text, TextFragment},
    input::keyboard::{KeyCode, KeyInput},
    Context, GameResult,
};
use glam::Vec2;
use rand::Rng;
use std::f32::consts::PI;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::SystemTime;
use std::panic;

// ἀρχὴ ἥμισυ παντός
struct Particle {
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
    size: f32,
}

impl Particle {
    fn update(&mut self, dt: f32) {
        self.position += self.velocity * dt;
        self.lifetime -= dt;
        self.size *= 0.95;
    }

    fn draw(&self, canvas: &mut Canvas, ctx: &Context) -> GameResult {
        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            self.position,
            self.size,
            0.1,
            Color::new(1.0, 1.0, 1.0, self.lifetime),
        )?;
        canvas.draw(&mesh, DrawParam::default());
        Ok(())
    }
}

// τὰ πάντα ῥεῖ
struct MainState {
    asteroids: Vec<Asteroid>,
    ship: Ship,
    bullets: Vec<Bullet>,
    game_over: bool,
    game_over_timer: f32,
    lives: i32,
    score: i32,
    displayed_score: i32,
    respawn_timer: f32,
    particles: Vec<Particle>,
    score_popup: Option<(String, Vec2, f32)>, // text, position, lifetime
    debug_info: String,
}

struct Asteroid {
    position: Vec2,
    velocity: Vec2,
    points: Vec<Vec2>,
    rotation: f32,
    rotation_speed: f32,
    size: AsteroidSize,
}

// μέτρον ἄριστον
#[derive(Clone, Copy)]
enum AsteroidSize {
    Large,   // Großer Asteroid
    Medium,  // Mittlerer Asteroid
    Small,   // Kleiner Asteroid
}

impl AsteroidSize {
    fn radius(&self) -> f32 {
        match self {
            AsteroidSize::Large => 80.0,
            AsteroidSize::Medium => 40.0,
            AsteroidSize::Small => 20.0,
        }
    }

    fn points(&self) -> i32 {
        match self {
            AsteroidSize::Large => 20,
            AsteroidSize::Medium => 50,
            AsteroidSize::Small => 100,
        }
    }

    fn next_size(&self) -> Option<AsteroidSize> {
        match self {
            AsteroidSize::Large => Some(AsteroidSize::Medium),
            AsteroidSize::Medium => Some(AsteroidSize::Small),
            AsteroidSize::Small => None,
        }
    }
}

// κίνησις πάντων
struct Ship {
    position: Vec2,
    velocity: Vec2,
    rotation: f32,
    thrust: bool,
    invulnerable: bool,
    invulnerable_timer: f32,
}

struct Bullet {
    position: Vec2,
    velocity: Vec2,
    lifetime: f32,
}

impl Asteroid {
    fn new_with_size(ctx: &Context, size: AsteroidSize) -> Self {
        let mut rng = rand::thread_rng();
        
        let (width, height) = ctx.gfx.drawable_size();
        let position = if rng.gen_bool(0.5) {
            Vec2::new(
                if rng.gen_bool(0.5) { 0.0 } else { width },
                rng.gen_range(0.0..height),
            )
        } else {
            Vec2::new(
                rng.gen_range(0.0..width),
                if rng.gen_bool(0.5) { 0.0 } else { height },
            )
        };

        let speed = rng.gen_range(50.0..150.0);
        let angle = rng.gen_range(0.0..2.0 * PI);
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let num_points = rng.gen_range(6..12);
        let mut points = Vec::new();
        let base_radius = size.radius();
        
        for i in 0..num_points {
            let angle = (i as f32 / num_points as f32) * 2.0 * PI;
            let radius = base_radius * rng.gen_range(0.5..1.5);
            points.push(Vec2::new(angle.cos() * radius, angle.sin() * radius));
        }

        Asteroid {
            position,
            velocity,
            points,
            rotation: 0.0,
            rotation_speed: rng.gen_range(-2.0..2.0),
            size,
        }
    }

    fn new(ctx: &Context) -> Self {
        Self::new_with_size(ctx, AsteroidSize::Large)
    }

    // σφαῖρος κυκλοτερής
    fn update(&mut self, ctx: &Context) {
        let dt = ctx.time.delta().as_secs_f32();
        self.position += self.velocity * dt;
        self.rotation += self.rotation_speed * dt;

        // Bildschirmgrenzen Wrapping (Screen edge wrapping)
        let (width, height) = ctx.gfx.drawable_size();
        if self.position.x < 0.0 {
            self.position.x = width;
        } else if self.position.x > width {
            self.position.x = 0.0;
        }
        if self.position.y < 0.0 {
            self.position.y = height;
        } else if self.position.y > height {
            self.position.y = 0.0;
        }
    }

    fn draw(&self, canvas: &mut Canvas, ctx: &Context) -> GameResult {
        let mut transformed_points = Vec::new();
        for point in &self.points {
            let rotated = Vec2::new(
                point.x * self.rotation.cos() - point.y * self.rotation.sin(),
                point.x * self.rotation.sin() + point.y * self.rotation.cos(),
            );
            transformed_points.push([
                rotated.x + self.position.x,
                rotated.y + self.position.y,
            ]);
        }

        let mesh = Mesh::new_polygon(
            ctx,
            DrawMode::stroke(2.0),
            &transformed_points,
            Color::WHITE,
        )?;
        canvas.draw(&mesh, DrawParam::default());
        Ok(())
    }

    fn split(&self) -> Option<Vec<Asteroid>> {
        let next_size = self.size.next_size()?;
        let mut rng = rand::thread_rng();
        let num_fragments = 2;
        let mut fragments = Vec::with_capacity(num_fragments);

        for _ in 0..num_fragments {
            let angle = rng.gen_range(0.0..2.0 * PI);
            let speed = self.velocity.length() * 1.5;
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);

            let mut asteroid = Asteroid {
                position: self.position,
                velocity,
                points: Vec::new(),
                rotation: rng.gen_range(0.0..2.0 * PI),
                rotation_speed: rng.gen_range(-3.0..3.0),
                size: next_size,
            };

            let num_points = rng.gen_range(6..12);
            let base_radius = next_size.radius();
            
            for i in 0..num_points {
                let angle = (i as f32 / num_points as f32) * 2.0 * PI;
                let radius = base_radius * rng.gen_range(0.5..1.5);
                asteroid.points.push(Vec2::new(angle.cos() * radius, angle.sin() * radius));
            }

            fragments.push(asteroid);
        }

        Some(fragments)
    }
}

// Neue Instanz des Raumschiffs erstellen (Create new ship instance)
impl Ship {
    fn new() -> Self {
        Ship {
            position: Vec2::ZERO,  // Wird in reset_position gesetzt
            velocity: Vec2::ZERO,
            rotation: 0.0,
            thrust: false,
            invulnerable: true,
            invulnerable_timer: 3.0,
        }
    }

    fn reset_position(&mut self, ctx: &Context) {
        let (width, height) = ctx.gfx.drawable_size();
        self.position = Vec2::new(width / 2.0, height / 2.0);
    }

    fn update(&mut self, ctx: &Context) {
        let dt = ctx.time.delta().as_secs_f32();
        
        if self.invulnerable {
            self.invulnerable_timer -= dt;
            if self.invulnerable_timer <= 0.0 {
                self.invulnerable = false;
            }
        }
        
        if self.thrust {
            let thrust_direction = Vec2::new(self.rotation.cos(), self.rotation.sin());
            self.velocity += thrust_direction * 200.0 * dt;
        }
        
        self.velocity *= 0.99;
        self.position += self.velocity * dt;
        
        let (width, height) = ctx.gfx.drawable_size();
        if self.position.x < 0.0 {
            self.position.x = width;
        } else if self.position.x > width {
            self.position.x = 0.0;
        }
        if self.position.y < 0.0 {
            self.position.y = height;
        } else if self.position.y > height {
            self.position.y = 0.0;
        }
    }

    fn draw(&self, canvas: &mut Canvas, ctx: &Context) -> GameResult {
        if self.invulnerable && (ctx.time.ticks() % 2 == 0) {
            return Ok(());
        }

        let points = [
            [40.0 * self.rotation.cos(), 40.0 * self.rotation.sin()],
            [
                -20.0 * self.rotation.cos() - 20.0 * self.rotation.sin(),
                -20.0 * self.rotation.sin() + 20.0 * self.rotation.cos(),
            ],
            [
                -20.0 * self.rotation.cos() + 20.0 * self.rotation.sin(),
                -20.0 * self.rotation.sin() - 20.0 * self.rotation.cos(),
            ],
        ];

        let transformed_points: Vec<[f32; 2]> = points
            .iter()
            .map(|[x, y]| [x + self.position.x, y + self.position.y])
            .collect();

        let mesh = Mesh::new_polygon(
            ctx,
            DrawMode::stroke(4.0),
            &transformed_points,
            Color::WHITE,
        )?;
        canvas.draw(&mesh, DrawParam::default());

        if self.thrust {
            let thrust_points = [
                [
                    -20.0 * self.rotation.cos() + 0.0 * self.rotation.sin(),
                    -20.0 * self.rotation.sin() - 0.0 * self.rotation.cos(),
                ],
                [
                    -40.0 * self.rotation.cos() - 10.0 * self.rotation.sin(),
                    -40.0 * self.rotation.sin() + 10.0 * self.rotation.cos(),
                ],
                [
                    -40.0 * self.rotation.cos() + 10.0 * self.rotation.sin(),
                    -40.0 * self.rotation.sin() - 10.0 * self.rotation.cos(),
                ],
            ];

            let transformed_thrust: Vec<[f32; 2]> = thrust_points
                .iter()
                .map(|[x, y]| [x + self.position.x, y + self.position.y])
                .collect();

            let thrust_mesh = Mesh::new_polygon(
                ctx,
                DrawMode::stroke(2.0),
                &transformed_thrust,
                Color::WHITE,
            )?;
            canvas.draw(&thrust_mesh, DrawParam::default());
        }

        Ok(())
    }

    fn shoot(&self) -> Bullet {
        let direction = Vec2::new(self.rotation.cos(), self.rotation.sin());
        Bullet {
            position: self.position + direction * 40.0,
            velocity: direction * 800.0 + self.velocity,
            lifetime: 1.0,
        }
    }
}

impl Bullet {
    fn update(&mut self, ctx: &Context) {
        let dt = ctx.time.delta().as_secs_f32();
        self.position += self.velocity * dt;
        self.lifetime -= dt;

        // Wrap around screen
        let (width, height) = ctx.gfx.drawable_size();
        if self.position.x < 0.0 {
            self.position.x = width;
        } else if self.position.x > width {
            self.position.x = 0.0;
        }
        if self.position.y < 0.0 {
            self.position.y = height;
        } else if self.position.y > height {
            self.position.y = 0.0;
        }
    }

    fn draw(&self, canvas: &mut Canvas, ctx: &Context) -> GameResult {
        let mesh = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            self.position,
            4.0,
            0.1,
            Color::WHITE,
        )?;
        canvas.draw(&mesh, DrawParam::default());
        Ok(())
    }
}

// Kollisionserkennung und Spiellogik
// ἀνάγκη δ᾽οὐδὲ θεοὶ μάχονται
impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        // Set up panic handler for crash reporting
        panic::set_hook(Box::new(|panic_info| {
            let timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            let crash_msg = format!(
                "\n[Crash Report {}]\nPanic occurred: {}\n",
                timestamp,
                panic_info
            );

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open("asteroids_crash.log")
            {
                let _ = writeln!(file, "{}", crash_msg);
            }
        }));

        let mut ship = Ship::new();
        ship.reset_position(ctx);
        
        let mut asteroids = Vec::new();
        for _ in 0..8 {
            asteroids.push(Asteroid::new(ctx));
        }
        Ok(MainState {
            asteroids,
            ship,
            bullets: Vec::new(),
            game_over: false,
            game_over_timer: 5.0,
            lives: 5,
            score: 0,
            displayed_score: 0,
            respawn_timer: 0.0,
            particles: Vec::new(),
            score_popup: None,
            debug_info: String::new(),
        })
    }

    // ἐκ τοῦ χάους
    fn create_explosion(&mut self, position: Vec2, size: f32) {
        let mut rng = rand::thread_rng();
        for _ in 0..50 {
            let angle = rng.gen_range(0.0..2.0 * PI);
            let speed = rng.gen_range(100.0..400.0);
            let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed);
            self.particles.push(Particle {
                position,
                velocity,
                lifetime: rng.gen_range(0.5..1.5),
                size: rng.gen_range(2.0..6.0) * size,
            });
        }
    }

    fn check_collisions(&mut self, ctx: &Context) {
        if self.game_over {
            return;
        }

        // Überprüfe Schiff-Asteroid Kollisionen (Check ship-asteroid collisions)
        if !self.ship.invulnerable {
            for asteroid in &self.asteroids {
                let distance = (asteroid.position - self.ship.position).length();
                if distance < asteroid.size.radius() + 15.0 {
                    self.create_explosion(self.ship.position, 1.0);
                    self.lives -= 1;
                    self.log_debug(&format!("Ship destroyed. Lives remaining: {}", self.lives));
                    
                    if self.lives <= 0 {
                        self.game_over = true;
                        self.game_over_timer = 5.0;
                        self.log_debug(&format!("Game Over. Final score: {}. Closing in 5 seconds.", self.score));
                    } else {
                        self.respawn_timer = 2.0;
                        let mut new_ship = Ship::new();
                        new_ship.reset_position(ctx);
                        self.ship = new_ship;
                    }
                    return;
                }
            }
        }

        // First, collect all valid collisions
        let mut valid_collisions: Vec<(usize, usize, i32, Vec2, AsteroidSize)> = Vec::new();
        {
            let bullets = &self.bullets;
            let asteroids = &self.asteroids;
            
            for (bullet_idx, bullet) in bullets.iter().enumerate() {
                for (asteroid_idx, asteroid) in asteroids.iter().enumerate() {
                    let distance = (asteroid.position - bullet.position).length();
                    if distance < asteroid.size.radius() {
                        if !valid_collisions.iter().any(|(_, a_idx, ..)| *a_idx == asteroid_idx) {
                            valid_collisions.push((
                                bullet_idx,
                                asteroid_idx,
                                asteroid.size.points(),
                                asteroid.position,
                                asteroid.size,
                            ));
                        }
                    }
                }
            }
        }

        // Process all valid collisions
        let mut new_asteroids = Vec::new();
        for (_, _, points, pos, size) in &valid_collisions {
            // Add score and create popup
            self.score += points;
            self.score_popup = Some((
                format!("+{}", points),
                *pos,
                1.0,
            ));
            
            // Create explosion effect
            self.create_explosion(*pos, size.radius() / 20.0);
        }

        // Handle asteroid splitting
        for (_, asteroid_idx, _, _, _) in &valid_collisions {
            if let Some(asteroid) = self.asteroids.get(*asteroid_idx) {
                if let Some(fragments) = asteroid.split() {
                    new_asteroids.extend(fragments);
                }
            }
        }

        // Remove hit bullets and asteroids (in reverse order)
        let mut indices: Vec<(usize, usize)> = valid_collisions.iter()
            .map(|(b, a, ..)| (*b, *a))
            .collect();
        indices.sort_by(|a, b| b.cmp(a));
        
        for (bullet_idx, asteroid_idx) in indices {
            if bullet_idx < self.bullets.len() {
                self.bullets.swap_remove(bullet_idx);
            }
            if asteroid_idx < self.asteroids.len() {
                self.asteroids.swap_remove(asteroid_idx);
                self.log_debug(&format!("Asteroid destroyed. Remaining: {}", self.asteroids.len()));
            }
        }

        // Add new asteroid fragments
        self.asteroids.extend(new_asteroids);
    }

    fn reset(&mut self, ctx: &mut Context) {
        self.asteroids.clear();
        for _ in 0..8 {
            self.asteroids.push(Asteroid::new(ctx));
        }
        let mut new_ship = Ship::new();
        new_ship.reset_position(ctx);
        self.ship = new_ship;
        self.bullets.clear();
        self.game_over = false;
        self.game_over_timer = 5.0;
        self.lives = 5;
        self.score = 0;
        self.displayed_score = 0;
        self.respawn_timer = 0.0;
        self.particles.clear();
        self.score_popup = None;
        self.debug_info.clear();
    }

    // γνῶσις
    fn log_debug(&mut self, msg: &str) {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        self.debug_info = format!("[{}] {}", timestamp, msg);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open("asteroids_debug.log")
        {
            let _ = writeln!(file, "{}", self.debug_info);
        }
    }
}

// Spielschleife und Updates
// πάντα χωρεῖ καὶ οὐδὲν μένει
impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ctx.time.delta().as_secs_f32();

        if self.game_over {
            self.game_over_timer -= dt;
            if self.game_over_timer <= 0.0 {
                ctx.request_quit();
            }
        }

        // Update score animation
        if self.displayed_score < self.score {
            self.displayed_score += ((self.score - self.displayed_score) as f32 * 10.0 * dt) as i32 + 1;
            if self.displayed_score > self.score {
                self.displayed_score = self.score;
            }
        }

        // Update score popup
        if let Some((_, _, ref mut lifetime)) = self.score_popup {
            *lifetime -= dt;
            if *lifetime <= 0.0 {
                self.score_popup = None;
            }
        }

        // Update particles
        self.particles.retain_mut(|particle| {
            particle.update(dt);
            particle.lifetime > 0.0
        });

        // Only update ship if not game over
        if !self.game_over {
            if self.respawn_timer > 0.0 {
                self.respawn_timer -= dt;
                if self.respawn_timer <= 0.0 {
                    let mut new_ship = Ship::new();
                    new_ship.reset_position(ctx);
                    self.ship = new_ship;
                }
            } else {
                self.ship.update(ctx);
            }
        }
        
        // Always update asteroids and bullets
        for asteroid in &mut self.asteroids {
            asteroid.update(ctx);
        }

        self.bullets.retain(|bullet| bullet.lifetime > 0.0);
        for bullet in &mut self.bullets {
            bullet.update(ctx);
        }

        self.check_collisions(ctx);
        Ok(())
    }

    // τὸ καλὸν
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);
        let (width, height) = ctx.gfx.drawable_size();
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        
        // Draw lives indicator on the left
        let lives_size = height / 15.0;
        let lives_text = Text::new(TextFragment::new("LIVES")
            .color(Color::WHITE)
            .scale(lives_size / 16.0));
        canvas.draw(&lives_text, DrawParam::default().dest(Vec2::new(40.0, 40.0)));

        // Draw ship icons for lives in a vertical arrangement
        for i in 0..self.lives {
            let points = [
                [30.0, 0.0],
                [-15.0, -15.0],
                [-15.0, 15.0],
            ];
            let transformed_points: Vec<[f32; 2]> = points
                .iter()
                .map(|[x, y]| [x + 70.0, y + 100.0 + (i as f32 * 50.0)])
                .collect();

            let mesh = Mesh::new_polygon(
                ctx,
                DrawMode::stroke(3.0),
                &transformed_points,
                Color::WHITE,
            )?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // Draw score popup with enhanced visibility
        if let Some((text, position, lifetime)) = &self.score_popup {
            let popup_scale = height / 30.0; // Proportional to screen height
            let popup_text = Text::new(TextFragment::new(text)
                .color(Color::new(1.0, 1.0, 1.0, *lifetime))
                .scale(popup_scale / 16.0));
            let text_dims = popup_text.dimensions(ctx).unwrap();
            
            // Draw shadow for better contrast
            let shadow_text = Text::new(TextFragment::new(text)
                .color(Color::new(0.0, 0.0, 0.0, *lifetime))
                .scale(popup_scale / 16.0));
            
            let pos = *position - Vec2::new(text_dims.w / 2.0, text_dims.h / 2.0);
            canvas.draw(&shadow_text, DrawParam::default().dest(pos + Vec2::new(2.0, 2.0)));
            canvas.draw(&popup_text, DrawParam::default().dest(pos));
        }

        // Draw game elements
        for asteroid in &self.asteroids {
            asteroid.draw(&mut canvas, ctx)?;
        }

        if !self.game_over && self.respawn_timer <= 0.0 {
            self.ship.draw(&mut canvas, ctx)?;
        }

        for bullet in &self.bullets {
            bullet.draw(&mut canvas, ctx)?;
        }

        // Draw particles
        for particle in &self.particles {
            particle.draw(&mut canvas, ctx)?;
        }

        if self.game_over {
            // Calculate size for game over text (half screen width)
            let base_scale = width / 400.0;
            
            let game_over_text = Text::new(
                TextFragment::new(format!(
                    "GAME OVER\nFinal Score: {:08}\nClosing in {:.1} seconds",
                    self.score,
                    self.game_over_timer
                ))
                .color(Color::WHITE)
                .scale(base_scale)
            );
            let text_dims = game_over_text.dimensions(ctx).unwrap();
            let pos = Vec2::new(
                center_x - text_dims.w / 2.0,
                center_y - text_dims.h / 2.0,
            );
            
            // Draw with shadow for better visibility
            let shadow_text = Text::new(
                TextFragment::new(format!(
                    "GAME OVER\nFinal Score: {:08}\nClosing in {:.1} seconds",
                    self.score,
                    self.game_over_timer
                ))
                .color(Color::new(0.0, 0.0, 0.0, 1.0))
                .scale(base_scale)
            );

            // Add pulsing effect to game over text
            let pulse = (ctx.time.ticks() as f32 * 0.01).sin() * 0.1 + 0.9;
            let shadow_offset = base_scale * 4.0;
            
            canvas.draw(&shadow_text, DrawParam::default()
                .dest(pos + Vec2::new(shadow_offset, shadow_offset))
                .scale(Vec2::new(pulse, pulse)));
            canvas.draw(&game_over_text, DrawParam::default()
                .dest(pos)
                .scale(Vec2::new(pulse, pulse)));
        }

        // Draw debug info if any
        if !self.debug_info.is_empty() {
            let debug_text = Text::new(TextFragment::new(&self.debug_info)
                .color(Color::new(0.7, 0.7, 0.7, 0.7))
                .scale(1.0));
            canvas.draw(&debug_text, DrawParam::default().dest(Vec2::new(10.0, height - 30.0)));
        }
        
        canvas.finish(ctx)?;
        Ok(())
    }

    // ἔλεγχος
    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult {
        match input.keycode {
            // Steuerungsbefehle (Control commands)
            Some(KeyCode::R) if self.game_over => self.reset(ctx),
            Some(KeyCode::Left) if !self.game_over => self.ship.rotation -= 0.1,
            Some(KeyCode::Right) if !self.game_over => self.ship.rotation += 0.1,
            Some(KeyCode::Up) if !self.game_over => self.ship.thrust = true,
            Some(KeyCode::Space) if !self.game_over => self.bullets.push(self.ship.shoot()),
            _ => (),
        }
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> GameResult {
        match input.keycode {
            Some(KeyCode::Up) => self.ship.thrust = false,
            _ => (),
        }
        Ok(())
    }
}

// ἡ ἀρχή
fn main() -> GameResult {
    let cb = ggez::ContextBuilder::new("asteroids", "ggez")
        .window_setup(ggez::conf::WindowSetup::default().title("Asteroids"))
        // Fenstergröße und Eigenschaften (Window size and properties)
        .window_mode(
            ggez::conf::WindowMode::default()
                .dimensions(1600.0, 1200.0)
                .resizable(true)
                .min_dimensions(800.0, 600.0)
        );
    
    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}
