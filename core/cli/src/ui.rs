use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::Print,
    terminal::{self, ClearType},
};
use std::io::{self, Result, Write};
use std::time::{Duration, Instant};

use simulation::ecs::World;
use simulation::resources::GameTime;

pub struct TermUI {
    city_name: String,
    last_update: Instant,
}

impl TermUI {
    pub fn new(city_name: String) -> Self {
        Self {
            city_name,
            last_update: Instant::now(),
        }
    }

    pub fn init(&self) -> Result<()> {
        terminal::enable_raw_mode()?;
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::Hide
        )?;
        Ok(())
    }

    pub fn cleanup(&self) -> Result<()> {
        execute!(
            io::stdout(),
            terminal::Clear(ClearType::All),
            cursor::Show
        )?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    pub fn render(&mut self, world: &World) -> Result<()> {
        let mut stdout = io::stdout();

        execute!(
            stdout,
            cursor::MoveTo(0, 0)
        )?;

        // Header
        execute!(
            stdout,
            Print("═".repeat(60)),
            Print("                    \r\n")
        )?;

        // City name
        execute!(
            stdout,
            Print(format!("  {}                              \r\n", self.city_name))
        )?;

        execute!(
            stdout,
            Print("═".repeat(60)),
            Print("                    \r\n\r\n")
        )?;

        // Time information
        if let Some(game_time) = world.get_resource::<GameTime>() {
            let day = game_time.current_day();
            let time_of_day = game_time.time_of_day();
            let hour = (time_of_day * 24.0) as u32;
            let minute = ((time_of_day * 24.0 * 60.0) % 60.0) as u32;

            execute!(
                stdout,
                Print(format!("  Day: {}                         \r\n", day + 1)),
                Print(format!("  Time: {:02}:{:02}                      \r\n", hour, minute))
            )?;

            // Status
            let status_text = if game_time.is_paused {
                "PAUSED"
            } else if game_time.speed > 60.0 {
                "FAST FORWARD"
            } else {
                "RUNNING"
            };

            execute!(
                stdout,
                Print(format!("  Status: {}                    \r\n", status_text))
            )?;

            execute!(
                stdout,
                Print(format!("  Speed: {:.1}x                   \r\n\r\n", game_time.speed / 60.0))
            )?;
        }

        // Controls
        execute!(
            stdout,
            Print("─".repeat(60)),
            Print("                    \r\n")
        )?;

        execute!(
            stdout,
            Print("  Controls:                         \r\n")
        )?;

        execute!(
            stdout,
            Print("    [0] Pause                              \r\n"),
            Print("    [1] Normal Speed (1x)                  \r\n"),
            Print("    [2] Fast Forward (2x)                  \r\n"),
            Print("    [5] Fast Forward (5x)                  \r\n"),
            Print("    [Q] Quit                               \r\n")
        )?;

        execute!(
            stdout,
            Print("─".repeat(60)),
            Print("                    \r\n")
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub fn handle_input(&self, world: &mut World) -> Result<bool> {
        if event::poll(Duration::from_millis(16))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('0') => {
                    if let Some(game_time) = world.get_resource_mut::<GameTime>() {
                        game_time.is_paused = true;
                    }
                }
                KeyCode::Char('1') => {
                    if let Some(game_time) = world.get_resource_mut::<GameTime>() {
                        game_time.is_paused = false;
                        game_time.speed = 1.0;
                    }
                }
                KeyCode::Char('2') => {
                    if let Some(game_time) = world.get_resource_mut::<GameTime>() {
                        game_time.is_paused = false;
                        game_time.speed = 2.0;
                    }
                }
                KeyCode::Char('5') => {
                    if let Some(game_time) = world.get_resource_mut::<GameTime>() {
                        game_time.is_paused = false;
                        game_time.speed = 5.0;
                    }
                }
                KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
                    return Ok(false);
                }
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn should_render(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(100) {
            self.last_update = now;
            true
        } else {
            false
        }
    }
}
