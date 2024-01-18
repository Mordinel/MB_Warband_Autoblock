mod mem;
mod proc;

use std::time::Duration;

use process_memory::Memory;
use crossterm::{
    execute,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

fn main() -> std::io::Result<()> {

    execute!(std::io::stdout(), EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let pid = proc::get_pid("mb_warband")?;
    println!("mb_warband PID: {pid}");

    let base_addr = proc::get_base_address(pid)?;
    println!("mb_warband base address: 0x{base_addr:x}");

    let handle = proc::get_handle(pid)?;
    println!("Got mb_warband process handle");

    #[cfg(target_os = "macos")]
    let autoblock_path = vec![0x47c2f4];

    #[cfg(not(target_os = "macos"))]
    let autoblock_path = vec![0x47c2f4];

    let autoblock_member = mem::resolve_pointer_path::<u32>(&handle, base_addr, &autoblock_path)?;

    let mut enable_autoblock = false;
    loop {
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                if should_break(&key_event) {
                    break;
                }
                match key_event.code {
                    KeyCode::Insert => {
                        enable_autoblock = !enable_autoblock;
                        if enable_autoblock {
                            println!("Enabled autoblock");
                        } else {
                            println!("Disabled autoblock");
                        }
                    },
                    _ => (),
                }
            }
        }
        if enable_autoblock {
            let _ = autoblock_member.write(&0);
        } else {
            let _ = autoblock_member.write(&1);
        }
    }

    terminal::disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn should_break(key_event: &KeyEvent) -> bool {
    if key_event.modifiers == KeyModifiers::CONTROL {
        match key_event.code {
            KeyCode::Char('c') | KeyCode::Char('d') => {
                return true;
            },
            _ => (),
        };
    }
    false
}
