mod mem;
mod proc;

use std::time::Duration;
use process_memory::Memory;
use crossterm::{
    terminal,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventKind},
};

fn main() -> std::io::Result<()> {
    setup_terminal()?;

    #[cfg(target_os = "macos")]
    let name = "Mount and Blade";
    #[cfg(target_os = "windows")]
    let name = "mb_warband.exe";

    let pid = proc::get_pid(name)
        .or_else(|e| {
            eprintln!("Failed to get pid for {name}: {e}\r");
            cleanup_terminal()?;
            Err(e)
        })?;
    println!("`{name}` PID: {pid}\r");

    let base_addr = proc::get_base_address(pid)
        .or_else(|e| {
            eprintln!("Failed to get base address for {pid}: {e}\r");
            cleanup_terminal()?;
            Err(e)
        })?;
    println!("`{name}` base address: 0x{base_addr:x}\r");

    let handle = proc::get_handle(pid)
        .or_else(|e| {
            eprintln!("Failed to get handle for {pid}: {e}\r");
            cleanup_terminal()?;
            Err(e)
        })?;
    println!("Got `{name}` process handle\r");

    #[cfg(target_os = "macos")]
    let autoblock_path = vec![0x14651bc];

    #[cfg(target_os = "windows")]
    let autoblock_path = vec![0x47c2f4];

    let autoblock_member = mem::resolve_pointer_path::<u32>(&handle, base_addr, &autoblock_path)
        .or_else(|e| {
            eprintln!("Failed to resolve pointer path {autoblock_path:?} in `{name}`: {e}\r");
            cleanup_terminal()?;
            Err(e)
        })?;

    println!("CTRL + C or CTRL + D to quit.\r");
    println!("END or ; to toggle autoblock.\r");

    let current_value = unsafe { autoblock_member.read() }
        .or_else(|e| {
            eprintln!("Failed to read from `autoblock_member` in `{name}`\r");
            cleanup_terminal()?;
            Err(e)
        })?;
    let mut enable_autoblock = current_value == 0;
    if enable_autoblock {
        println!("Autoblock already enabled\r");
    } else {
        println!("Autoblock NOT enabled\r");
    }
    loop {
        if proc::get_pid(name).is_err() {
            eprintln!("Game closed, exiting\r");
            break;
        }

        if event::poll(Duration::from_millis(5))
            .or_else(|e| {
                eprintln!("Error when polling for console events: {e}\r");
                cleanup_terminal()?;
                Err(e)
            })? {

            if let Event::Key(key_event) = event::read()
                .or_else(|e| {
                    eprintln!("Error when reading console event: {e}\r");
                    cleanup_terminal()?;
                    Err(e)
                })? {

                if should_break(&key_event) {
                    break;
                }
                match key_event.code {
                    KeyCode::End | KeyCode::Char(';') => {
                        if key_event.kind == KeyEventKind::Press {
                            enable_autoblock = !enable_autoblock;
                            if enable_autoblock {
                                println!("Enabled autoblock\r");
                            } else {
                                println!("Disabled autoblock\r");
                            }
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
        std::thread::sleep(Duration::from_millis(5));
    }

    cleanup_terminal()?;
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

fn setup_terminal() -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    Ok(())
}

fn cleanup_terminal() -> std::io::Result<()> {
    terminal::disable_raw_mode()?;
    Ok(())
}

