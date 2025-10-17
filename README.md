# Balthazar

A skeleton Bevy game project ready for development.

## Features

- Basic Bevy setup with default plugins
- Avian2D physics integration with rope simulation
- 2D camera and player entity
- Player attached to central pole with realistic rope physics
- WASD movement controls (physics-based)
- Organized asset directory structure
- Top-down view with zero gravity

## Controls

- **W/A/S/D**: Apply forces to move the player (connected by rope to pole)
- Player movement is constrained by rope physics - you can swing around the pole!
- **Spacebar**: Attach/detach cord from poles
  - If cord is attached to a pole: disconnect it
  - If cord is not attached: attach to closest pole within range (100 units)
- **Shift**: Hold to retract the cord length
- **Escape**: Close the game window

## Running the Game

```bash
cargo run
```

## Building for Release

```bash
cargo build --release
```

## Project Structure

```
balthazar/
├── src/
│   └── main.rs          # Main game logic
├── assets/
│   ├── textures/        # Image files
│   ├── sounds/          # Audio files
│   └── fonts/           # Font files
├── Cargo.toml           # Project dependencies
└── README.md
```

## Next Steps

- Add physics bodies to entities using Avian2D (RigidBody, Collider components)
- Add sprites and textures to the `assets/textures/` directory
- Implement game entities and systems
- Add audio using files in `assets/sounds/`
- Create UI elements with custom fonts from `assets/fonts/`

Built with [Bevy](https://bevyengine.org/) 0.17 and [Avian2D](https://github.com/Jondolf/avian) 0.4
