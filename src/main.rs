use std::env;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};

use chrono::Local;
use image::GenericImageView;
use pam::Authenticator;
use rand::seq::SliceRandom;
use rusttype::{Font, Scale, PositionedGlyph};
use users::get_current_username;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::randr;
use x11rb::rust_connection::RustConnection;
use x11rb::{COPY_FROM_PARENT, NONE};

// --- THEME & CONSTANTS ---
const COLOR_BASE: u32 = 0xFF0B0F18;
const COLOR_MANTLE: u32 = 0xE6080C14; 
const COLOR_TEAL: u32 = 0xFF0FB5B3;
const COLOR_ICE: u32 = 0xFFA8E6F1;
const COLOR_TEXT: u32 = 0xFFF7F9FB;
const COLOR_SUBTEXT: u32 = 0xFFAAB5BF;
const COLOR_ERROR: u32 = 0xFFE27878;
const COLOR_WARNING: u32 = 0xFFE2A478;

const FUNNY_PHRASES: &[&str] = &[
    "It works on my machine...",
    "Have you tried turning it off and on again?",
    "Layer 8 Issue Detected.",
    "Git blame: You.",
    "Unexpected token: You.",
    "SEGFAULT: User not found.",
    "404: Password not found.",
    "Nice try, script kiddie.",
    "sudo make me a sandwich?",
    "Compiling... just kidding, wrong password.",
];

#[derive(Clone, Copy)]
struct Monitor {
    x: i16,
    y: i16,
    w: u16,
    h: u16,
}

struct ArcticLock {
    conn: RustConnection,
    window: Window,
    width: u16,
    height: u16,
    gcontext: Gcontext,
    buffer: Vec<u32>,
    font: Font<'static>,
    monitors: Vec<Monitor>,
    
    // State
    user: String,
    password: String,
    status_msg: String,
    status_color: u32,
    funny_phrase: String,
    
    // Animation
    shake_intensity: i32,
    blink_timer: u32,
    shift_pressed: bool,
    
    // Resources
    background: Option<image::DynamicImage>,
}

impl ArcticLock {
    fn new(font_path: &str, bg_path: Option<String>) -> Result<Self, Box<dyn std::error::Error>> {
        // 1. Setup X11
        let (conn, screen_num) = RustConnection::connect(None)?;
        let screen = &conn.setup().roots[screen_num];
        let root = screen.root;
        
        let width = screen.width_in_pixels;
        let height = screen.height_in_pixels;

        // 2. DETECT MONITORS (Fixed Logic)
        let mut monitors = Vec::new();
        
        // Split the request and the reply to handle Error types correctly
        if let Ok(res_cookie) = randr::get_screen_resources_current(&conn, root) {
            if let Ok(res) = res_cookie.reply() {
                for &crtc in &res.crtcs {
                    if let Ok(info_cookie) = randr::get_crtc_info(&conn, crtc, res.config_timestamp) {
                        if let Ok(info) = info_cookie.reply() {
                            if info.width > 0 && info.mode != 0 {
                                monitors.push(Monitor {
                                    x: info.x,
                                    y: info.y,
                                    w: info.width,
                                    h: info.height,
                                });
                            }
                        }
                    }
                }
            }
        }

        if monitors.is_empty() {
            monitors.push(Monitor { x: 0, y: 0, w: width, h: height });
        }

        println!("Detected {} monitor(s).", monitors.len());

        // 3. Create Window
        let win_id = conn.generate_id()?;
        let win_aux = CreateWindowAux::new()
            .background_pixel(screen.black_pixel)
            .override_redirect(1) 
            .event_mask(EventMask::KEY_PRESS | EventMask::KEY_RELEASE);

        conn.create_window(
            COPY_FROM_PARENT as u8,
            win_id,
            root,
            0, 0, width, height,
            0,
            WindowClass::INPUT_OUTPUT,
            screen.root_visual,
            &win_aux,
        )?;

        // 4. Graphics Context
        let gc_id = conn.generate_id()?;
        conn.create_gc(gc_id, win_id, &CreateGCAux::new())?;

        // 5. Map & Grab
        conn.map_window(win_id)?;
        conn.flush()?;
        
        let mut grabbed = false;
        for i in 0..50 {
            let kb = conn.grab_keyboard(
                true, win_id, Time::CURRENT_TIME, GrabMode::ASYNC, GrabMode::ASYNC
            );
            let ptr = conn.grab_pointer(
                true, win_id, 
                EventMask::BUTTON_PRESS | EventMask::BUTTON_RELEASE | EventMask::POINTER_MOTION,
                GrabMode::ASYNC, GrabMode::ASYNC, 
                win_id, NONE, Time::CURRENT_TIME
            );

            if kb.is_ok() && ptr.is_ok() {
                // Check if reply status is SUCCESS
                let kb_status = kb.unwrap().reply().map(|r| r.status).unwrap_or(GrabStatus::ALREADY_GRABBED);
                let ptr_status = ptr.unwrap().reply().map(|r| r.status).unwrap_or(GrabStatus::ALREADY_GRABBED);
                
                if kb_status == GrabStatus::SUCCESS && ptr_status == GrabStatus::SUCCESS {
                    grabbed = true;
                    break;
                }
            }
            thread::sleep(Duration::from_millis(50));
            if i % 10 == 0 { println!("Attempting to grab input..."); }
        }

        if !grabbed {
            eprintln!("CRITICAL: Failed to grab inputs!");
        }

        // 6. Load Font
        let font_data = std::fs::read(font_path).expect("Failed to read font file");
        let font = Font::try_from_vec(font_data).expect("Error constructing font");

        // 7. Load Background
        let background = if let Some(path) = bg_path {
            if Path::new(&path).exists() {
                println!("Loading background: {}", path);
                image::open(path).ok().map(|img| img.resize_exact(width as u32, height as u32, image::imageops::FilterType::Lanczos3))
            } else {
                None
            }
        } else {
            None
        };

        // 8. Get User
        let user = get_current_username()
            .map(|u| u.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Unknown".into());

        Ok(Self {
            conn,
            window: win_id,
            width,
            height,
            gcontext: gc_id,
            buffer: vec![COLOR_BASE; width as usize * height as usize], 
            font,
            user,
            monitors,
            password: String::new(),
            status_msg: "Enter Password".to_string(),
            status_color: COLOR_SUBTEXT,
            funny_phrase: String::new(),
            shake_intensity: 0,
            blink_timer: 0,
            shift_pressed: false,
            background,
        })
    }

    // --- DRAWING ENGINE ---

    fn clear_buffer(&mut self) {
        if let Some(ref bg) = self.background {
            for (i, pixel) in bg.pixels().enumerate() {
                 if i < self.buffer.len() {
                    let rgba = pixel.2;
                    self.buffer[i] = ((rgba[3] as u32) << 24) | ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | (rgba[2] as u32);
                 }
            }
        } else {
            self.buffer.fill(COLOR_BASE);
        }
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: u32) {
        let alpha = (color >> 24) & 0xFF;
        for dy in 0..h {
            for dx in 0..w {
                let px = x + dx as i32;
                let py = y + dy as i32;
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    let idx = (py as usize * self.width as usize) + px as usize;
                    if alpha == 255 {
                        self.buffer[idx] = color;
                    } else {
                        let bg = self.buffer[idx];
                        let inv_a = 255 - alpha;
                        let r = (((color >> 16) & 0xFF) * alpha + ((bg >> 16) & 0xFF) * inv_a) / 255;
                        let g = (((color >> 8) & 0xFF) * alpha + ((bg >> 8) & 0xFF) * inv_a) / 255;
                        let b = ((color & 0xFF) * alpha + (bg & 0xFF) * inv_a) / 255;
                        self.buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    }

    fn draw_text(&mut self, text: &str, x: i32, y: i32, scale: f32, color: u32) -> u32 {
        let scale = Scale::uniform(scale);
        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(x as f32, y as f32 + v_metrics.ascent);
        let glyphs: Vec<PositionedGlyph> = self.font.layout(text, scale, offset).collect();
        let mut width = 0;

        for glyph in glyphs {
            if let Some(bb) = glyph.pixel_bounding_box() {
                width = std::cmp::max(width, bb.max.x as u32);
                glyph.draw(|gx, gy, v| {
                    let px = gx as i32 + bb.min.x;
                    let py = gy as i32 + bb.min.y;
                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        let idx = (py as usize * self.width as usize) + px as usize;
                        let alpha = (v * 255.0) as u32;
                        let bg = self.buffer[idx];
                        let inv_a = 255 - alpha;
                        let r = (((color >> 16) & 0xFF) * alpha + ((bg >> 16) & 0xFF) * inv_a) / 255;
                        let g = (((color >> 8) & 0xFF) * alpha + ((bg >> 8) & 0xFF) * inv_a) / 255;
                        let b = ((color & 0xFF) * alpha + (bg & 0xFF) * inv_a) / 255;
                        self.buffer[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                });
            }
        }
        width.saturating_sub(x as u32)
    }

    fn measure_text(&self, text: &str, scale: f32) -> i32 {
        let scale = Scale::uniform(scale);
        let glyphs: Vec<PositionedGlyph> = self.font.layout(text, scale, rusttype::point(0.0, 0.0)).collect();
        glyphs.last().map(|g| g.pixel_bounding_box().map(|b| b.max.x).unwrap_or(0)).unwrap_or(0)
    }

    // --- GAME LOOP & LOGIC ---

    fn render(&mut self) {
        self.clear_buffer();

        // 1. Update Animation State
        let mut shake_offset = 0;
        if self.shake_intensity > 0 {
            self.shake_intensity -= 1;
            shake_offset = (self.shake_intensity as f32 * 0.5).sin() as i32 * 10;
        }
        self.blink_timer += 1;

        // 2. Draw UI for EACH monitor
        let monitors = self.monitors.clone();
        for m in monitors {
            self.draw_ui_at_monitor(m, shake_offset);
        }

        // 3. Send to X Server
        self.present();
    }

    fn draw_ui_at_monitor(&mut self, m: Monitor, shake_offset: i32) {
        let cx = (m.x as i32) + (m.w as i32 / 2) + shake_offset;
        let cy = (m.y as i32) + (m.h as i32 / 2);

        // Date & Time
        let now = Local::now();
        let time_str = now.format("%H:%M").to_string();
        let date_str = now.format("%A, %B %d").to_string();

        let time_w = self.measure_text(&time_str, 120.0);
        self.draw_text(&time_str, cx - (time_w / 2), cy - 300, 120.0, COLOR_TEXT);
        
        let date_w = self.measure_text(&date_str, 40.0);
        self.draw_text(&date_str, cx - (date_w / 2), cy - 190, 40.0, COLOR_ICE);

        // Login Box
        let box_w = 450;
        let box_h = 250;
        let box_x = cx - (box_w / 2);
        let box_y = cy - (box_h / 2);

        self.draw_rect(box_x, box_y, box_w as u32, box_h as u32, COLOR_MANTLE);
        self.draw_rect(box_x, box_y, box_w as u32, 2, COLOR_TEAL); // Top
        self.draw_rect(box_x, box_y + box_h - 2, box_w as u32, 2, COLOR_TEAL); // Bottom

        // User Label
        let user_str = format!("User: {}", self.user);
        let user_w = self.measure_text(&user_str, 32.0);
        self.draw_text(&user_str, cx - (user_w / 2), cy - 60, 32.0, COLOR_TEXT);

        // Password Input (Left Aligned)
        let input_start_x = cx - 180; 

        if self.password.is_empty() {
            let placeholder = "Start Typing...";
            let ph_w = self.measure_text(placeholder, 32.0);
            self.draw_text(placeholder, cx - (ph_w / 2), cy + 10, 32.0, COLOR_SUBTEXT);

            if (self.blink_timer / 15) % 2 == 0 {
                self.draw_rect(input_start_x, cy + 15, 2, 25, COLOR_TEAL);
            }
        } else {
            let pass_str = "• ".repeat(self.password.len());
            self.draw_text(&pass_str, input_start_x, cy + 10, 32.0, COLOR_ICE);
            let pass_w = self.measure_text(&pass_str, 32.0);
            
            if (self.blink_timer / 15) % 2 == 0 {
                self.draw_rect(input_start_x + pass_w + 2, cy + 15, 2, 25, COLOR_TEAL);
            }
        }

        // Status Messages
        let status_msg_clone = self.status_msg.clone();
        let status_color = self.status_color;
        let status_w = self.measure_text(&status_msg_clone, 20.0);
        self.draw_text(&status_msg_clone, cx - (status_w / 2), cy + 60, 20.0, status_color);

        if !self.funny_phrase.is_empty() {
             let funny_phrase_clone = self.funny_phrase.clone();
             let phrase_w = self.measure_text(&funny_phrase_clone, 20.0);
             self.draw_text(&funny_phrase_clone, cx - (phrase_w / 2), cy + 90, 20.0, COLOR_WARNING);
        }
    }

    fn present(&self) {
        let data: &[u8] = unsafe {
            std::slice::from_raw_parts(self.buffer.as_ptr() as *const u8, self.buffer.len() * 4)
        };

        let stride = (self.width as usize) * 4;
        let rows_per_chunk = (64 * 1024) / stride;
        let rows_per_chunk = std::cmp::max(1, rows_per_chunk);

        let mut y = 0;
        while y < self.height {
            let height_remaining = self.height - y;
            let chunk_h = std::cmp::min(height_remaining, rows_per_chunk as u16);
            let start = (y as usize) * stride;
            let end = start + (chunk_h as usize) * stride;
            
            if end > data.len() { break; } 
            
            let chunk_data = &data[start..end];
            self.conn.put_image(
                ImageFormat::Z_PIXMAP,
                self.window,
                self.gcontext,
                self.width,
                chunk_h,
                0,
                y as i16,
                0,
                24,
                chunk_data
            ).unwrap();

            y += chunk_h;
        }
    }

    fn authenticate(&mut self) -> bool {
        let mut authenticator = Authenticator::with_password("login").expect("PAM Init failed");
        authenticator.get_handler().set_credentials(&self.user, &self.password);
        authenticator.authenticate().is_ok()
    }

    fn run(&mut self) {
        loop {
            let start = Instant::now();
            while let Some(event) = self.conn.poll_for_event().unwrap() {
                match event {
                    x11rb::protocol::Event::KeyPress(ev) => self.handle_key(ev.detail, true),
                    x11rb::protocol::Event::KeyRelease(ev) => self.handle_key(ev.detail, false),
                    _ => {}
                }
            }
            self.render();
            let elapsed = start.elapsed();
            if elapsed < Duration::from_millis(16) {
                thread::sleep(Duration::from_millis(16) - elapsed);
            }
        }
    }

    fn handle_key(&mut self, keycode: u8, pressed: bool) {
        if keycode == 50 || keycode == 62 {
            self.shift_pressed = pressed;
            return;
        }
        if !pressed { return; }

        self.status_msg = "Authenticating...".to_string();
        self.status_color = COLOR_SUBTEXT;
        self.funny_phrase.clear();

        match keycode {
            36 => { // Enter
                self.render(); 
                if self.authenticate() {
                    std::process::exit(0);
                } else {
                    self.password.clear();
                    self.status_msg = "Access Denied".to_string();
                    self.status_color = COLOR_ERROR;
                    self.shake_intensity = 20;
                    self.funny_phrase = FUNNY_PHRASES.choose(&mut rand::thread_rng()).unwrap().to_string();
                }
            }
            22 => { self.password.pop(); } 
            9 => { self.password.clear(); } 
            _ => {
                if let Some(ch) = keycode_to_char(keycode, self.shift_pressed) {
                    self.password.push(ch);
                }
            }
        }
    }
}

fn keycode_to_char(code: u8, shift: bool) -> Option<char> {
    let base = match code {
        10..=19 => Some("1234567890".chars().nth((code - 10) as usize).unwrap()),
        24..=33 => Some("qwertyuiop".chars().nth((code - 24) as usize).unwrap()),
        38..=46 => Some("asdfghjkl".chars().nth((code - 38) as usize).unwrap()),
        52..=58 => Some("zxcvbnm".chars().nth((code - 52) as usize).unwrap()),
        65 => Some(' '),
        20 => Some('-'), 21 => Some('='),
        34 => Some('['), 35 => Some(']'),
        51 => Some('\\'), 
        47 => Some(';'), 48 => Some('\''),
        59 => Some(','), 60 => Some('.'), 61 => Some('/'),
        _ => None,
    };
    if let Some(c) = base {
        if shift {
            return Some(match c {
                '1' => '!', '2' => '@', '3' => '#', '4' => '$', '5' => '%',
                '6' => '^', '7' => '&', '8' => '*', '9' => '(', '0' => ')',
                '-' => '_', '=' => '+', '[' => '{', ']' => '}', '\\' => '|',
                ';' => ':', '\'' => '"', ',' => '<', '.' => '>', '/' => '?',
                _ => c.to_ascii_uppercase(),
            });
        }
        return Some(c);
    }
    None
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: arctic-lock <path_to_font.ttf> [path_to_background.png]");
        std::process::exit(1);
    }
    
    let font_path = &args[1];
    let bg_path = if args.len() > 2 { Some(args[2].clone()) } else { None };

    match ArcticLock::new(font_path, bg_path) {
        Ok(mut lock) => lock.run(),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
