use crossterm::{
    ExecutableCommand,
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{
        Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
        enable_raw_mode,
    },
};
use simulation::City;
use std::io::{Write, stdout};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    // Enable raw mode for keyboard input
    enable_raw_mode().expect("Failed to enable raw mode");

    // Enter alternate screen and hide cursor
    let mut out = stdout();
    out.execute(EnterAlternateScreen)
        .expect("Failed to enter alternate screen");
    out.execute(Hide).expect("Failed to hide cursor");

    // Ensure cleanup happens on exit
    let result = std::panic::catch_unwind(|| {
        run_simulation();
    });

    // Cleanup - reuse the same stdout handle
    let _ = out.execute(Show);
    let _ = out.execute(LeaveAlternateScreen);
    let _ = disable_raw_mode();

    if let Err(e) = result {
        eprintln!("Simulation panicked: {:?}", e);
    }
}

fn run_simulation() {
    // Create a new city
    let mut city = City::new("Test City");

    let target_fps = 10.0;
    let frame_duration = Duration::from_secs_f64(1.0 / target_fps);
    let mut last_time = Instant::now();
    let mut accumulator = Duration::ZERO;
    let base_timestep = 1.0 / target_fps;

    let mut paused = false;
    let mut speed_multiplier = 1.0;

    // Initial render
    render(&city, paused, speed_multiplier);

    loop {
        // Check for keyboard input (non-blocking)
        if event::poll(Duration::from_millis(0)).unwrap_or(false)
            && let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
        {
            match (code, modifiers) {
                // Ctrl+C to exit
                (KeyCode::Char('e'), KeyModifiers::NONE) => {
                    break;
                }
                // Ctrl+Space to pause/unpause
                (KeyCode::Char(' '), KeyModifiers::NONE) => {
                    paused = !paused;
                }
                // Ctrl+D to toggle 2x speed
                (KeyCode::Char('1'), KeyModifiers::NONE) => {
                    speed_multiplier = 1.0;
                }
                (KeyCode::Char('2'), KeyModifiers::NONE) => {
                    speed_multiplier = 2.0;
                }
                (KeyCode::Char('3'), KeyModifiers::NONE) => {
                    speed_multiplier = 3.0;
                }
                _ => {}
            }
        }

        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_time);
        last_time = current_time;

        if !paused {
            accumulator += elapsed;

            while accumulator >= frame_duration {
                let timestep = base_timestep * speed_multiplier;
                city.update(timestep);
                accumulator -= frame_duration;
            }
        }

        render(&city, paused, speed_multiplier);
        thread::sleep(frame_duration);
    }
}

fn render(city: &City, paused: bool, speed: f64) {
    let mut out = stdout();

    // Clear screen and move cursor to top-left
    out.execute(Clear(ClearType::All)).unwrap();
    out.execute(MoveTo(0, 0)).unwrap();

    // Use \r\n for proper line breaks in raw mode
    print!("{}\r\n\r\n", city.display_status());

    let status_text = if paused { "PAUSED" } else { "RUNNING" };
    print!("Status: {}\r\n", status_text);
    print!("Speed: {}x\r\n\r\n", speed);

    print!("  SPACE - Pause/Unpause\r\n");
    print!("  1 - 1x speed\r\n");
    print!("  2 - 2x speed\r\n");
    print!("  3 - 3x speed\r\n");
    print!("  e - Exit\r\n");

    // Flush to ensure all output is displayed
    out.flush().unwrap();
}
