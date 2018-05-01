extern crate libremarkable;

use libremarkable::framebuffer::common::*;

use libremarkable::appctx;

use libremarkable::framebuffer::{FramebufferDraw, FramebufferRefresh};

use libremarkable::input::{gpio, multitouch, wacom};

extern crate num_complex;
use num_complex::Complex;

fn on_button_press(app: &mut appctx::ApplicationContext, input: gpio::GPIOEvent) {
    println!("on_button_press");

    let (_btn, new_state) = match input {
        gpio::GPIOEvent::Press { button } => (button, true),
        gpio::GPIOEvent::Unpress { button } => (button, false),
        _ => return,
    };

    // Ignoring the unpressed event
    if !new_state {
        return;
    }

    let framebuffer = app.get_framebuffer_ref();

    julia(app);

    framebuffer.full_refresh(
        waveform_mode::WAVEFORM_MODE_AUTO,
        display_temp::TEMP_USE_REMARKABLE_DRAW,
        dither_mode::EPDC_FLAG_USE_DITHERING_DRAWING,
        0,
        true,
    );
}

fn on_wacom_input(_app: &mut appctx::ApplicationContext, _input: wacom::WacomEvent) {
    println!("on_wacom_input");
}
fn on_touch_handler(_app: &mut appctx::ApplicationContext, _input: multitouch::MultitouchEvent) {
    println!("on_touch_handler");
}

fn julia(app: &mut appctx::ApplicationContext) {
    let framebuffer = app.get_framebuffer_ref();

    let width = framebuffer.var_screen_info.xres;
    let height = framebuffer.var_screen_info.yres;

    // constants to tweak for appearance
    let iterations = 32;
    let cx = -0.9;
    let cy = 0.27015;

    let inner_height = height as f32;
    let inner_width = width as f32;

    for x in 0..width {
        for y in 0..height {
            let inner_y = y as f32;
            let inner_x = x as f32;

            let mut zx = 3.0 * (inner_y - 0.5 * inner_height) / (inner_height);
            let mut zy = 2.0 * (inner_x - 0.5 * inner_width) / (inner_width);

            let mut i = iterations;

            while zx * zx + zy * zy < 4.0 && i > 1 {
                let tmp = zx * zx - zy * zy + cx;
                zy = 2.0 * zx * zy + cy;
                zx = tmp;
                i -= 1;
            }

            let r = (i << 3) as u8;
            let g = (i << 5) as u8;
            let b = (i << 4) as u8;

            put_pixel(app, x as usize, y as usize, rgb_to_native(r, g, b));
        }
    }
    println!("Julia drawing is done!");
}

fn mandelbrot(app: &mut appctx::ApplicationContext) {
    let framebuffer = app.get_framebuffer_ref();

    let width = framebuffer.var_screen_info.xres;
    let height = framebuffer.var_screen_info.yres;

    // constants to tweak for appearance
    let iterations = 32u16; //256u16; //256u16 >> 5;
    let cxmin = -2f32;
    let cxmax = 1f32;

    let ysize = (cxmax - cxmin) * ((height as f32) / (width as f32));
    let cymin = -1.0 * ysize / 2.0;
    let cymax = ysize / 2.0;

    let scalex = (cxmax - cxmin) / width as f32;
    let scaley = (cymax - cymin) / height as f32;

    for x in 0..width {
        for y in 0..height {
            let cx = cxmin + x as f32 * scalex;
            let cy = cymin + y as f32 * scaley;

            let c = Complex::new(cx, cy);
            let mut z = Complex::new(0f32, 0f32);

            let mut i = 0;
            for t in 0..iterations {
                if z.norm() > 2.0 {
                    break;
                }
                z = z * z + c;
                i = t;
            }

            let r = (i << 3) as u8;
            let g = (i << 5) as u8;
            let b = (i << 4) as u8;

            put_pixel(app, x as usize, y as usize, rgb_to_native(r, g, b));
        }
    }

    println!("Mandelbrot drawing is done!");
}

fn main() {
    let mut app: appctx::ApplicationContext =
        appctx::ApplicationContext::new(on_button_press, on_wacom_input, on_touch_handler);

    app.clear(true);

    let framebuffer = app.get_framebuffer_ref();

    framebuffer.draw_text(
        1600,
        445,
        String::from("Drawing is in progress..."),
        50,
        color::BLACK,
    );
    framebuffer.draw_text(
        1650,
        511,
        String::from("(press any hw key to draw Julia set)"),
        25,
        color::GRAY(80),
    );

    framebuffer.full_refresh(
        waveform_mode::WAVEFORM_MODE_AUTO,
        display_temp::TEMP_USE_REMARKABLE_DRAW,
        dither_mode::EPDC_FLAG_USE_DITHERING_DRAWING,
        0,
        true,
    );

    mandelbrot(&mut app);
    //julia(&mut app);

    framebuffer.full_refresh(
        waveform_mode::WAVEFORM_MODE_AUTO,
        display_temp::TEMP_USE_REMARKABLE_DRAW,
        dither_mode::EPDC_FLAG_USE_DITHERING_DRAWING,
        0,
        true,
    );

    app.dispatch_events(true, true, true);
}

fn rgb_to_native(red: u8, green: u8, blue: u8) -> u16 {
    let r: u16 = (((red >> 3) & 0x1f) as u16) << 11;
    let g: u16 = (((green >> 2) & 0x3f) as u16) << 5;
    let b: u16 = ((blue >> 3) & 0x1f) as u16;

    (r | g | b)
}

fn put_pixel(app: &mut appctx::ApplicationContext, x: usize, y: usize, native: u16) {
    let framebuffer = app.get_framebuffer_ref();

    let w = framebuffer.var_screen_info.xres as usize;
    let h = framebuffer.var_screen_info.yres as usize;
    if y >= h || x >= w {
        return;
    }

    let begin = framebuffer.frame.data() as *mut u8;

    let line_length = framebuffer.fix_screen_info.line_length as usize;
    let bytespp = (framebuffer.var_screen_info.bits_per_pixel / 8) as usize;
    let curr_index = ((y) * line_length + (x) * bytespp) as isize;

    unsafe {
        *(begin.offset(curr_index + 0)) = (native) as u8;
        *(begin.offset(curr_index + 1)) = (native >> 8) as u8;
    }
}
