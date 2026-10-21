//! Menu bar icons, drawn in code as template images (black + alpha, so macOS
//! tints them for light and dark menu bars). A moon means the Mac sleeps
//! normally; a sun means sleep is disabled.

use tray_icon::Icon;

const SIZE: u32 = 36; // 18 pt at 2x

pub fn moon() -> Icon {
    render(|x, y| {
        let outer = circle(x, y, 18.0, 18.0, 13.0);
        let bite = circle(x, y, 25.0, 12.0, 10.5);
        outer.max(-bite)
    })
}

pub fn sun() -> Icon {
    render(|x, y| {
        let mut d = circle(x, y, 18.0, 18.0, 6.5);
        for i in 0..8 {
            let a = i as f32 * std::f32::consts::FRAC_PI_4;
            let (s, c) = a.sin_cos();
            let ray = segment(x, y, 18.0 + c * 10.0, 18.0 + s * 10.0, 18.0 + c * 15.0, 18.0 + s * 15.0) - 1.4;
            d = d.min(ray);
        }
        d
    })
}

/// Rasterise a signed distance function (negative inside) with 1px antialiasing.
fn render(sdf: impl Fn(f32, f32) -> f32) -> Icon {
    let mut rgba = Vec::with_capacity((SIZE * SIZE * 4) as usize);
    for py in 0..SIZE {
        for px in 0..SIZE {
            let d = sdf(px as f32 + 0.5, py as f32 + 0.5);
            let alpha = (0.5 - d).clamp(0.0, 1.0);
            rgba.extend_from_slice(&[0, 0, 0, (alpha * 255.0).round() as u8]);
        }
    }
    Icon::from_rgba(rgba, SIZE, SIZE).expect("icon buffer matches its size")
}

fn circle(x: f32, y: f32, cx: f32, cy: f32, r: f32) -> f32 {
    ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r
}

fn segment(x: f32, y: f32, ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    let (pax, pay, bax, bay) = (x - ax, y - ay, bx - ax, by - ay);
    let h = ((pax * bax + pay * bay) / (bax * bax + bay * bay)).clamp(0.0, 1.0);
    ((pax - bax * h).powi(2) + (pay - bay * h).powi(2)).sqrt()
}
