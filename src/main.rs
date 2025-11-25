//! Minimal reproduction of egui-winit clipboard segfault on Wayland shutdown
//!
//! This demonstrates a SOUNDNESS BUG in egui-winit:
//!
//! 1. egui_winit::Clipboard::new() is a SAFE function
//! 2. Internally it calls smithay_clipboard::Clipboard::new() which is UNSAFE
//! 3. smithay-clipboard requires: "display must remain valid for as long as
//!    Clipboard object is alive"
//! 4. egui-winit CANNOT guarantee this - the caller controls window lifetime
//!
//! The segfault occurs when:
//! 1. egui-winit Clipboard is created (which spawns smithay's background thread)
//! 2. The window/Wayland display is destroyed BEFORE clipboard is dropped
//! 3. smithay's background thread tries to use the now-invalid display
//!
//! This is 100% safe Rust code - NO unsafe blocks - yet it segfaults.
//!
//! Run with: cargo run --release
//! The window will auto-close after 1 second.

use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::raw_window_handle::{HasDisplayHandle, RawDisplayHandle};
use winit::window::{Window, WindowId};

struct App {
    // Window declared BEFORE clipboard - this means window drops LAST.
    // This is the "wrong" order that triggers the bug, but it's completely
    // reasonable code that a user might write. Nothing warns about this.
    //
    // NOTE: Swapping the order of `window` and `clipboard` fields would
    // "fix" the segfault because Rust drops fields in declaration order.
    // But that's exactly the point - a safe API should not segfault based
    // on field ordering! The compiler gives no warning about this.
    window: Option<Arc<Window>>,
    clipboard: Option<egui_winit::clipboard::Clipboard>,
    start_time: Option<Instant>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = Arc::new(
                event_loop
                    .create_window(Window::default_attributes().with_title("Will auto-close in 1s"))
                    .unwrap(),
            );

            // Get the display handle
            let raw_display = window.display_handle().ok().map(|h| h.as_raw());

            // Check if we're on Wayland
            if let Some(RawDisplayHandle::Wayland(_)) = raw_display {
                // Create egui-winit clipboard using the SAFE API
                // Note: NO unsafe block here! This is the soundness bug.
                let clipboard = egui_winit::clipboard::Clipboard::new(raw_display);
                self.clipboard = Some(clipboard);
                println!("Created egui-winit Clipboard (safe API, no unsafe block!)");
            } else {
                println!("Not running on Wayland, segfault won't occur");
            }

            self.window = Some(window);
            self.start_time = Some(Instant::now());

            // Request continuous polling so we can check the timer
            event_loop.set_control_flow(ControlFlow::Poll);
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // Auto-close after 1 second
        if let Some(start) = self.start_time {
            if start.elapsed() >= Duration::from_secs(1) {
                println!("Auto-closing window after 1 second...");
                println!("Watch for SEGFAULT - this is 100% safe Rust code!");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            println!("Window close requested, exiting...");
            event_loop.exit();
        }
    }
}

fn main() {
    println!("Demonstrating egui-winit soundness bug");
    println!("======================================");
    println!();
    println!("This program uses ONLY safe Rust - no unsafe blocks.");
    println!("Yet it will segfault on Wayland due to egui-winit wrapping");
    println!("an unsafe API (smithay-clipboard) in a safe interface.");
    println!();
    println!("Window will auto-close in 1 second...");
    println!();

    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        window: None,
        clipboard: None,
        start_time: None,
    };

    event_loop.run_app(&mut app).unwrap();

    println!("Event loop exited, dropping App...");
    println!("(window drops first, then clipboard tries to use invalid display)");
}
