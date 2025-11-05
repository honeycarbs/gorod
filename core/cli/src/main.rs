mod ui;

use std::time::{Duration, Instant};
use simulation::{World, GameTime, clock_system};
use ui::TermUI;
use std::io::Result;

fn main() -> Result<()> {
    let mut world = World::new();
    world.insert_resource(GameTime::new(1440.0));
    
    let mut ui = TermUI::new("New Springfield".to_string());
    ui.init()?;
    
    let target_fps = 10.0;
    let frame_duration = Duration::from_secs_f64(1.0 / target_fps);
    let mut last_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let base_timestep = 1.0 / target_fps as f32;
    
    let mut running = true;
    
    while running {
        // Handle input
        running = ui.handle_input(&mut world)?;
        
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);
        last_time = current_time;
        
        // Get game time to check if paused
        let is_paused = world.get_resource::<GameTime>()
            .map(|gt| gt.is_paused)
            .unwrap_or(false);
        
        if !is_paused {
            accumulator += elapsed;
            
            while accumulator >= frame_duration {
                // Update systems with fixed timestep
                clock_system(&mut world, base_timestep);
                accumulator -= frame_duration;
            }
        }
        
        // Render
        if ui.should_render() {
            ui.render(&world)?;
        }
        
        // Sleep to target frame rate
        std::thread::sleep(frame_duration);
    }
    
    ui.cleanup()?;
    Ok(())
}