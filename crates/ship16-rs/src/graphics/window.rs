use pixels::{Pixels, SurfaceTexture};
use std::{sync::Arc, time::Instant};
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow},
    keyboard::{Key, KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use super::palette::PALETTE;
use crate::{FRAME_DURATION, emulator::Emulator};

const WIDTH: u32 = 64;
const HEIGHT: u32 = 64;

pub struct App {
    emulator: Emulator,
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    next_frame: Instant,
}

impl App {
    pub fn new(emulator: Emulator) -> Self {
        Self {
            emulator,
            window: None,
            pixels: None,
            next_frame: Instant::now(),
        }
    }
}

impl App {
    fn render(&mut self) -> Result<(), pixels::Error> {
        let frame = self.pixels.as_mut().unwrap().frame_mut();

        for (i, &index) in self.emulator.vram().iter().enumerate() {
            let [r, g, b] = PALETTE[index as usize];
            let offset = i * 4;

            frame[offset] = r;
            frame[offset + 1] = g;
            frame[offset + 2] = b;
            frame[offset + 3] = 255;
        }

        self.pixels.as_mut().unwrap().render()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("Ship-16")
                        .with_inner_size(winit::dpi::LogicalSize::new(640.0, 640.0)),
                )
                .expect("Failed to create window"),
        );

        let size = window.inner_size();

        let surface = SurfaceTexture::new(size.width, size.height, window.clone());

        let pixels = Pixels::new(WIDTH, HEIGHT, surface).expect("Failed to create pixel buffer");

        self.pixels = Some(pixels);
        self.window = Some(window);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }

                if let Some(pixels) = self.pixels.as_mut() {
                    pixels
                        .resize_surface(size.width, size.height)
                        .expect("Failed to resize pixel buffer");
                }
            }
            WindowEvent::RedrawRequested => {
                self.emulator.run_frame();
                if let Err(err) = self.render() {
                    eprintln!("Failed to render frame: {err}");
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key:
                            PhysicalKey::Code(
                                KeyCode::Digit0
                                | KeyCode::Digit1
                                | KeyCode::Digit2
                                | KeyCode::Digit3
                                | KeyCode::Digit4
                                | KeyCode::Digit5
                                | KeyCode::Digit6
                                | KeyCode::Digit7
                                | KeyCode::Digit8
                                | KeyCode::Digit9
                                | KeyCode::KeyA
                                | KeyCode::KeyB
                                | KeyCode::KeyC
                                | KeyCode::KeyD
                                | KeyCode::KeyE
                                | KeyCode::KeyF,
                            ),
                        logical_key: Key::Character(k),
                        ..
                    },
                ..
            } => {
                let key_char = k.to_ascii_lowercase().chars().next().unwrap();
                let key_index = match key_char {
                    '0'..='9' => key_char as u8 - b'0',
                    'a'..='f' => key_char as u8 - b'a' + 10,
                    _ => return,
                };

                if state == ElementState::Released {
                    self.emulator.key_up(key_index);
                } else {
                    self.emulator.key_down(key_index);
                }
            }
            WindowEvent::Focused(false) => {
                for key in 0..16 {
                    self.emulator.key_up(key);
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        event_loop.set_control_flow(ControlFlow::WaitUntil(self.next_frame));

        if Instant::now() >= self.next_frame {
            if let Some(window) = &self.window {
                window.request_redraw();
            }

            self.next_frame += FRAME_DURATION;

            if self.next_frame < Instant::now() {
                self.next_frame = Instant::now() + FRAME_DURATION;
            }
        }
    }
}
