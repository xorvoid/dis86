use sdl2;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};
use sdl2::pixels::PixelFormatEnum;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
struct Color(u32);

impl Color {
  fn from_rgb(r: u8, g: u8, b: u8) -> Color {
    let c = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    Color(c)
  }
}

// ---------------------------------------------------------------------------
// Software framebuffer
// ---------------------------------------------------------------------------

struct Framebuffer {
  pixels: Vec<Color>,
}

impl Framebuffer {
  fn new() -> Self {
    Self { pixels: vec![Color(0); WIDTH * HEIGHT] }
  }

  fn clear(&mut self, color: Color) {
    self.pixels.fill(color);
  }

  fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
    if x < 0 || x >= WIDTH as i32 || y < 0 || y >= HEIGHT as i32 {
      return;
    }
    self.pixels[y as usize * WIDTH + x as usize] = color;
  }

  fn as_bytes(&self) -> &[u8] {
    // SAFETY: Color is repr(transparent) over u32; slice is valid for its length.
    unsafe {
      std::slice::from_raw_parts(
        self.pixels.as_ptr() as *const u8,
        self.pixels.len() * std::mem::size_of::<Color>(),
      )
    }
  }
}

// ---------------------------------------------------------------------------
// Renderer — owns canvas, texture_creator, and texture together.
//
// With the `unsafe_textures` feature Texture has no lifetime parameter, so
// this struct is self-contained and can be passed around freely.
// The invariant we must uphold manually: `texture` must be dropped before
// `texture_creator`, which must be dropped before `canvas`. The reverse-
// declaration order of struct fields ensures Rust drops them in that order.
// ---------------------------------------------------------------------------

pub struct App {
  sdl:     sdl2::Sdl,
  events:  sdl2::EventPump,
  //video:   sdl2::VideoSubsystem,
  //window: sdl2::video::Window,
  //canvas:  sdl2::render::WindowCanvas,
  //texture: sdl2::render::Texture<'_>,
  renderer: Renderer,
  fb: Framebuffer,
}


struct Renderer {
  canvas: Canvas<Window>,
  // Declared after canvas so it is dropped first.
  // Stored only to keep the creator alive; texture must not outlive it.
  #[allow(dead_code)]
  texture_creator: TextureCreator<WindowContext>,
  // Declared last so it is dropped first of all three.
  texture: Texture,
}

impl Renderer {
  fn new(canvas: Canvas<Window>) -> Self {
    // SAFETY: we keep texture_creator alive as long as texture lives (same struct).
    let texture_creator = canvas.texture_creator();
    let texture = texture_creator
      .create_texture_streaming(PixelFormatEnum::RGB888, WIDTH as u32, HEIGHT as u32)
      .unwrap();
    Self { canvas, texture_creator, texture }
  }

  fn upload(&mut self, fb: &Framebuffer) {
    self.texture
      .update(None, fb.as_bytes(), WIDTH * std::mem::size_of::<Color>())
      .unwrap();
  }

  fn present(&mut self) {
    self.canvas.clear();
    self.canvas.copy(&self.texture, None, None).unwrap();
    self.canvas.present();
  }
}

impl App {
  pub fn new() -> App {
    let sdl = sdl2::init().unwrap();
    let events = sdl.event_pump().unwrap();
    let video = sdl.video().unwrap();

    let window = video
      .window("emu86", WIDTH as u32, HEIGHT as u32)
      .position_centered()
      .build()
      .unwrap();

    let canvas = window.into_canvas().accelerated().build().unwrap();
    let mut renderer = Renderer::new(canvas);

    let mut fb = Framebuffer::new();
    fb.clear(Color::from_rgb(0x10, 0x10, 0x20));

    App {
      sdl,
      events,
      renderer,
      fb,
    }
  }

  pub fn handle_events(&mut self) -> bool {
    for event in self.events.poll_iter() {
      match event {
        Event::Quit { .. } => return true,
        Event::KeyDown { keycode: Some(Keycode::Escape), .. } => return true,
        _ => {}
      }
    }
    false
  }

  pub fn update(&mut self) -> Result<bool, String> {
    self.renderer.upload(&self.fb);
    self.renderer.present();

    let quit = self.handle_events();

    Ok(quit)
  }

  // pub fn run(&mut self) {
  //   let sdl = sdl2::init().unwrap();
  //   let video = sdl.video().unwrap();

  //   let window = video
  //     .window("VGA Framebuffer – Triangle (Rust)", WIDTH as u32, HEIGHT as u32)
  //     .position_centered()
  //     .build()
  //     .unwrap();

  //   let mut canvas = window.into_canvas().accelerated().build().unwrap();
  //   let texture_creator = canvas.texture_creator();

  //   let mut texture = texture_creator
  //     .create_texture_streaming(PixelFormatEnum::RGB888, WIDTH as u32, HEIGHT as u32)
  //     .unwrap();

  //   // --- Software framebuffer (our "VGA memory") ---
  //   //let mut fb = Framebuffer::new();
  // }
}
