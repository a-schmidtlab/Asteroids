# Asteroids 

Axel is learnin Rust

"Space is big. Really big. You just won't believe how vastly, hugely, mind-bogglingly big it is."

A little implementation of Asteroids arcade , written in Rust using the GGEZ game engine.

## Features

* Smooth spaceship controls with thrust and rotation
* Dynamic asteroid splitting mechanics
* Particle effects for explosions
* Score system with visual popups
* Lives system with ship respawning
* Game over sequence with final score display
* Debug logging system
* Crash reporting
* Fullscreen support with resizable window

## Controls

* Left Arrow: Rotate counterclockwise
* Right Arrow: Rotate clockwise
* Up Arrow: Thrust forward
* Space: Fire bullets
* R: Reset game (when game over)

## Technical Details

"Time is an illusion. Gameplay doubly so."

The game is built with:
- Rust 
- GGEZ game engine for graphics and input handling
- Glam for vector mathematics
- Rand for random number generation

## Installation

"Don't Panic!"

1. Make sure you have Rust installed
2. Clone this repository
3. Run `cargo build` to compile
4. Run `cargo run` to start the game

## Architecture

The game is structured around the main components:

* `MainState`: Core game state management
* `Ship`: Player spacecraft physics and rendering
* `Asteroid`: Asteroid behavior and splitting mechanics
* `Bullet`: Projectile physics
* `Particle`: Explosion effect system

## Performance

"The ships hung in the sky in much the same way that bricks don't."

The game features:
- collision detection
- Smooth particle systems
- Memory-conscious object pooling
- Frame-independent physics

## Contributing

"So long, and thanks for all the commits!"

Feel free to:
- Report bugs
- Suggest features
- Submit pull requests
- Improve documentation

## License

MIT License - See LICENSE file for details 
