use crate::theme::Color;


pub fn okabe_ito_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(0, 114, 178),   // Blue
        Color::from_rgb(213, 94, 0),    // Vermillion
        Color::from_rgb(0, 158, 115),   // Bluish Green
        Color::from_rgb(230, 159, 0),   // Orange
        Color::from_rgb(86, 180, 233),  // Sky Blue
        Color::from_rgb(240, 228, 66),  // Yellow
        Color::from_rgb(0, 0, 0),       // Black
        Color::from_rgb(204, 121, 167), // Reddish Purple
    ]
}

pub fn color_brewer_set1_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(228, 26, 28),   // Red
        Color::from_rgb(55, 126, 184),  // Blue
        Color::from_rgb(77, 175, 74),   // Green
        Color::from_rgb(152, 78, 163),  // Purple
        Color::from_rgb(255, 127, 0),   // Orange
        Color::from_rgb(255, 255, 51),  // Yellow
        Color::from_rgb(166, 86, 40),   // Brown
        Color::from_rgb(247, 129, 191), // Pink
        Color::from_rgb(153, 153, 153), // Grey
    ]
}

pub fn color_brewer_set2_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(102, 194, 165), // Teal
        Color::from_rgb(252, 141, 98),  // Salmon
        Color::from_rgb(141, 160, 203), // Lavender
        Color::from_rgb(231, 138, 195), // Pink
        Color::from_rgb(166, 216, 84),  // Light Green
        Color::from_rgb(255, 217, 47),  // Light Yellow
        Color::from_rgb(229, 196, 148), // Beige
        Color::from_rgb(179, 179, 179), // Light Grey
    ]
}

pub fn color_brewer_set3_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(141, 211, 199),
        Color::from_rgb(255, 255, 179),
        Color::from_rgb(190, 186, 218),
        Color::from_rgb(251, 128, 114),
        Color::from_rgb(128, 177, 211),
        Color::from_rgb(253, 180, 98),
        Color::from_rgb(179, 222, 105),
        Color::from_rgb(252, 205, 229),
        Color::from_rgb(217, 217, 217),
        Color::from_rgb(188, 128, 189),
        Color::from_rgb(204, 235, 197),
        Color::from_rgb(255, 237, 111),
    ]
}

pub fn viridis_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(68, 1, 84),
        Color::from_rgb(71, 44, 122),
        Color::from_rgb(59, 81, 139),
        Color::from_rgb(44, 113, 142),
        Color::from_rgb(33, 144, 141),
        Color::from_rgb(39, 173, 129),
        Color::from_rgb(92, 200, 99),
        Color::from_rgb(170, 220, 50),
        Color::from_rgb(253, 231, 37),
    ]
}

pub fn plasma_palette() -> Vec<Color> {
    vec![
        Color::from_rgb(13, 8, 135),
        Color::from_rgb(75, 3, 161),
        Color::from_rgb(125, 3, 168),
        Color::from_rgb(168, 34, 150),
        Color::from_rgb(203, 70, 121),
        Color::from_rgb(229, 107, 93),
        Color::from_rgb(248, 148, 65),
        Color::from_rgb(253, 195, 40),
        Color::from_rgb(240, 249, 33),
    ]
}

pub fn discrete_palette(n: usize) -> Vec<Color> {
    if n <= 8 {
        return okabe_ito_palette()[0..n].to_vec();
    }

    // Fall back on equal spacing in CIELAB space for larger n
    let mut colors = Vec::with_capacity(n);
    for i in 0..n {
        let hue = (i as f64) / (n as f64) * 360.0;
        let (r, g, b) = cielab_to_rgb(70.0, 50.0, hue);
        colors.push(Color::from_rgb(r, g, b));
    }
    colors
}

fn cielab_to_rgb(l: f64, a: f64, b: f64) -> (u8, u8, u8) {
    // Convert CIE L*a*b* to RGB
    // Reference: http://www.easyrgb.com/en/math.php
    
    // Step 1: Convert L*a*b* to XYZ
    let mut y = (l + 16.0) / 116.0;
    let mut x = a / 500.0 + y;
    let mut z = y - b / 200.0;
    
    // Apply inverse f function
    let delta = 6.0 / 29.0;
    let delta_cubed = delta * delta * delta;
    
    x = if x > delta {
        x.powf(3.0)
    } else {
        3.0 * delta * delta * (x - 4.0 / 29.0)
    };
    
    y = if y > delta {
        y.powf(3.0)
    } else {
        3.0 * delta * delta * (y - 4.0 / 29.0)
    };
    
    z = if z > delta {
        z.powf(3.0)
    } else {
        3.0 * delta * delta * (z - 4.0 / 29.0)
    };
    
    // Reference white point D65
    x *= 0.95047;
    y *= 1.00000;
    z *= 1.08883;
    
    // Step 2: Convert XYZ to linear RGB (sRGB color space)
    let r_linear = x *  3.2406 + y * -1.5372 + z * -0.4986;
    let g_linear = x * -0.9689 + y *  1.8758 + z *  0.0415;
    let b_linear = x *  0.0557 + y * -0.2040 + z *  1.0570;
    
    // Step 3: Apply gamma correction (sRGB companding)
    let gamma = |c: f64| -> f64 {
        if c <= 0.0031308 {
            12.92 * c
        } else {
            1.055 * c.powf(1.0 / 2.4) - 0.055
        }
    };
    
    let r = gamma(r_linear);
    let g = gamma(g_linear);
    let b = gamma(b_linear);
    
    // Step 4: Clamp to [0, 1] and convert to [0, 255]
    let clamp = |c: f64| -> u8 {
        (c.max(0.0).min(1.0) * 255.0).round() as u8
    };
    
    (clamp(r), clamp(g), clamp(b))
}