# Balthazar Game Systems

This document outlines the various systems implemented in Balthazar and their features.

## Core Systems

### 1. Player Movement System
**File:** `src/main.rs` - `move_player()`

**Features:**
- WASD-based movement controls
- Physics-based force application
- Isometric movement mapping (W/S for Y-axis, A/D for X-axis)
- Velocity-based movement (300.0 units/sec)
- Can be toggled on/off via UI

**Controls:**
- `W` - Move forward (positive Y)
- `A` - Move left (negative X)  
- `S` - Move backward (negative Y)
- `D` - Move right (positive X)

### 2. Cord Physics System
**Files:** `src/cord_system.rs`, `src/main.rs`

**Core Features:**
- Dynamic rope simulation with segmented physics
- Retractable/extendable cord length (50-500 units)
- Attachment/detachment to poles
- Realistic joint-based physics using Avian2D

**Sub-systems:**

#### Cord Retraction (`handle_cord_retraction`)
- **Trigger:** Hold Shift key
- **Retraction speed:** 300 units/sec
- **Extension speed:** 80 units/sec (automatic when player pulls)
- **Dynamic segment management:** Adds/removes cord segments based on length
- **Minimum segments:** Always maintains at least 2 segments

#### Cord Attachment (`handle_cord_attachment`) 
- **Trigger:** Spacebar
- **Attachment range:** 100 units
- **Auto-detects closest pole within range**
- **Can attach/detach from any pole**
- **Visual/audio feedback via console messages**

**Components:**
- `CordSegment` - Individual rope segments
- `CordSystem` - Resource managing entire cord state
- `CordAttachment` - Physics attachment point on player

### 3. Camera System
**File:** `src/main.rs`

**Features:**

#### Camera Follow (`camera_follow_player`)
- Smoothly tracks player position
- Maintains fixed Z-position for 2D view
- Real-time position synchronization
- Can be toggled on/off

#### Camera Zoom (`camera_zoom`)
- Mouse wheel zoom controls
- Zoom factor: 1.1x per scroll increment
- Zoom limits: 0.1x to 10x scale
- Supports both line and pixel scroll units
- Orthographic projection for consistent scaling

### 4. Player Rotation System
**File:** `src/main.rs` - `rotate_player()`

**Features:**
- Dynamic rotation based on movement and input
- Context-aware facing direction:
  - **Player-controlled movement:** Faces movement direction
  - **Physics-driven movement:** Faces opposite to pull direction
  - **Input vs. physics conflict:** Faces intended input direction
- Smooth rotation interpolation (4.0 rad/sec for controlled, 2.0 rad/sec for physics)
- Velocity threshold: Only rotates when moving >10 units/sec
- Can be toggled on/off

### 5. UI Management System
**File:** `src/main.rs`

**Features:**

#### System Toggle Interface (`setup_ui`, `update_ui`)
- Real-time system enable/disable controls
- Visual toggle buttons (Green=ON, Red=OFF)
- Semi-transparent overlay panel
- Controls for all major systems:
  - Player Movement
  - Cord Systems  
  - Camera Follow
  - Camera Zoom
  - Player Rotation

#### UI Components
- `SystemToggles` - Resource tracking system states
- `ToggleButton` - Component for interactive buttons
- Real-time visual feedback
- Positioned in top-right corner

### 6. Entity Spawn System
**File:** `src/main.rs` - `setup()`

**Features:**

#### Player Entity
- Circular mesh (20px radius) 
- Blue coloration
- Dynamic physics body with damping
- Visual enhancements:
  - White eye with black pupil (directional indicator)
  - Brown backpack (visual detail)
  - Separate physics attachment point for cord

#### Pole System
- Multiple pole entities (5 total)
- Varied positioning across game world
- Visual design:
  - Tall vertical shafts (15x80 units)
  - Horizontal crossbars (60x8 units) 
  - Color-coded (center pole darker)
- Static physics bodies with collision

#### Cord Generation
- Dynamic segment creation based on max length
- Interconnected segments via distance joints
- Configurable segment properties:
  - Length: 20 units per segment
  - Visual size: 8x8 units
  - Physics damping for stability

## System Architecture

### Resource Management
- `CordSystem` - Central cord state and configuration
- `SystemToggles` - UI control states
- `Gravity(Vec2::ZERO)` - Disabled gravity for top-down gameplay

### Component Hierarchy
- `Player` - Main player entity marker
- `Pole` - Static attachment points  
- `CordSegment` - Individual rope physics segments
- `CordAttachment` - Player's cord connection point

### Physics Integration
- **Engine:** Avian2D 0.4
- **Joint types:** DistanceJoint with configurable limits
- **Body types:** Dynamic (player, cord), Static (poles)
- **Collision:** Circle colliders for all dynamic entities

## System Interactions

### Movement ↔ Physics
- Player input applies forces to dynamic body
- Cord physics constrains movement radius
- Rotation system responds to both input and physics

### Cord ↔ Environment  
- Attachment system queries pole positions
- Distance calculations for range checking
- Dynamic joint creation/destruction

### UI ↔ Systems
- Toggle states control system execution
- Real-time enable/disable functionality
- Visual feedback for current states

### Camera ↔ Player
- Position tracking with smooth following
- Zoom controls independent of player state
- Maintains consistent view perspective

## Performance Considerations

- **Damping values** tuned for stability vs. responsiveness
- **Joint limits** configured to minimize physics jitter
- **Segment management** optimized to add/remove only when necessary
- **UI updates** only on interaction events
- **System toggles** allow selective performance optimization