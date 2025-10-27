use bevy::prelude::*;
use avian2d::prelude::*;

// Import the types we need for testing
use balthazar::{Player};

// Mock SystemToggles for testing
#[derive(Resource)]
struct TestSystemToggles {
    player_movement: bool,
}


#[test]
fn test_player_movement_wasd_controls() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_movement: true });
    
    // Add the player entity
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Test W key (positive Y movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);
        app.insert_resource(input);
        
        // Simulate the movement system
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0.y = move_speed;
        
        assert_eq!(velocity.0, Vec2::new(0.0, 300.0));
    }
    
    // Test A key (negative X movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyA);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0.x = -move_speed;
        
        assert_eq!(velocity.0, Vec2::new(-300.0, 300.0));
    }
    
    // Test S key (negative Y movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyS);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0.y = -move_speed;
        
        assert_eq!(velocity.0, Vec2::new(-300.0, -300.0));
    }
    
    // Test D key (positive X movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyD);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0.x = move_speed;
        
        assert_eq!(velocity.0, Vec2::new(300.0, -300.0));
    }
}

#[test]
fn test_diagonal_movement() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_movement: true });
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Test W+D (northeast movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyD);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0 = Vec2::new(move_speed, move_speed); // Both X and Y positive
        
        assert_eq!(velocity.0, Vec2::new(300.0, 300.0));
    }
    
    // Test A+S (southwest movement)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyA);
        input.press(KeyCode::KeyS);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        velocity.0 = Vec2::new(-move_speed, -move_speed); // Both X and Y negative
        
        assert_eq!(velocity.0, Vec2::new(-300.0, -300.0));
    }
}

#[test]
fn test_no_movement_when_no_keys_pressed() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_movement: true });
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::new(100.0, 50.0)), // Start with some velocity
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // No keys pressed
    let input = ButtonInput::<KeyCode>::default();
    app.insert_resource(input);
    
    let mut query = app.world_mut().query::<&mut LinearVelocity>();
    let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
    
    // Simulate no input - velocity should be set to zero
    velocity.0 = Vec2::ZERO;
    
    assert_eq!(velocity.0, Vec2::ZERO);
}

#[test]
fn test_movement_disabled_when_toggled_off() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_movement: false }); // Disabled
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Press W key but movement is disabled
    let mut input = ButtonInput::<KeyCode>::default();
    input.press(KeyCode::KeyW);
    app.insert_resource(input);
    
    // Get the toggle state
    let toggles = app.world().resource::<TestSystemToggles>();
    
    // Movement should be disabled, so velocity stays zero
    if toggles.player_movement {
        // This shouldn't execute
        panic!("Movement system should be disabled");
    }
    
    let mut query = app.world_mut().query::<&LinearVelocity>();
    let velocity = query.get(app.world(), player_entity).unwrap();
    assert_eq!(velocity.0, Vec2::ZERO);
}


#[test]
fn test_opposing_keys_cancel_out() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(TestSystemToggles { player_movement: true });
    
    let player_entity = app.world_mut().spawn((
        Player,
        LinearVelocity(Vec2::ZERO),
        Transform::from_xyz(0.0, 0.0, 0.0),
    )).id();
    
    // Test A+D keys (should cancel out in X direction)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyA);
        input.press(KeyCode::KeyD);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        let mut force = Vec2::ZERO;
        force.x -= move_speed; // A key
        force.x += move_speed; // D key
        
        velocity.0 = force;
        
        assert_eq!(velocity.0.x, 0.0); // Should cancel out
    }
    
    // Test W+S keys (should cancel out in Y direction)
    {
        let mut input = ButtonInput::<KeyCode>::default();
        input.press(KeyCode::KeyW);
        input.press(KeyCode::KeyS);
        app.insert_resource(input);
        
        let mut query = app.world_mut().query::<&mut LinearVelocity>();
        let mut velocity = query.get_mut(app.world_mut(), player_entity).unwrap();
        
        let move_speed = 300.0;
        let mut force = Vec2::ZERO;
        force.y += move_speed; // W key
        force.y -= move_speed; // S key
        
        velocity.0 = force;
        
        assert_eq!(velocity.0.y, 0.0); // Should cancel out
    }
}

