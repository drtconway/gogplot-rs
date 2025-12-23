// Utility functions for scale operations

/// Generate axis breaks using Wilkinson's Extended algorithm.
/// Returns a vector of break positions given a data range and desired number of breaks.
pub fn extended_breaks(domain: (f64, f64), n: usize) -> Vec<f64> {
    extended_breaks_weighted(domain, n, 1.0)
}

/// Generate axis breaks with configurable density weighting.
/// Higher density_weight values more strongly prefer getting exactly n breaks.
pub fn extended_breaks_weighted(domain: (f64, f64), n: usize, density_weight: f64) -> Vec<f64> {
    extended_breaks_weighted_clamped(domain, n, density_weight, false)
}

/// Generate axis breaks with optional clamping to domain.
/// When clamped=true, ensures all breaks are within [min, max].
pub fn extended_breaks_weighted_clamped(domain: (f64, f64), n: usize, density_weight: f64, clamp: bool) -> Vec<f64> {
    // Nice numbers to use for step sizes
    const Q: [f64; 5] = [1.0, 5.0, 2.0, 2.5, 4.0];

    let (min, max) = domain;
    if n < 2 {
        return vec![min, max];
    }

    // Handle degenerate case: single value
    if (min - max).abs() < 1e-10 {
        // Create a symmetric range around the value
        if min.abs() < 1e-10 {
            // Value is ~0, create range around 0
            return vec![-1.0, 0.0, 1.0];
        } else {
            // Create range ±10% around the value
            let range = min.abs() * 0.1;
            return vec![min - range, min, min + range];
        }
    }
    let range = max - min;
    let mut best_score = std::f64::NEG_INFINITY;
    let mut best = vec![min, max];

    for &q in &Q {
        let w = (range / (n as f64 - 1.0)) / q;
        let step = q * 10f64.powf(w.log10().floor());
        let start = (min / step).floor() * step;
        let end = (max / step).ceil() * step;
        let mut breaks = Vec::new();
        let mut x = start;
        while x <= end + 1e-10 {
            breaks.push(x);
            x += step;
        }
        
        // If clamping is requested, filter breaks to be within domain
        if clamp {
            breaks.retain(|&b| b >= min && b <= max);
            // If we filtered out all breaks, skip this candidate
            if breaks.is_empty() {
                continue;
            }
        }
        
        // Score: coverage, simplicity, density, legibility (simplified)
        // Normalize coverage by the data range to make it comparable with other terms
        let coverage_raw = (min - breaks[0]).abs() + (breaks.last().unwrap() - max).abs();
        let coverage = coverage_raw / range; // Normalize to [0, ~2] range
        let simplicity = if q == 1.0 { 1.0 } else { 0.5 };
        let density = (breaks.len() as f64 - n as f64).abs();
        let score = -coverage - density * density_weight + simplicity;
        if score > best_score {
            log::info!("Extended breaks candidate: {:?}, score: {}", breaks, score);
            best_score = score;
            best = breaks;
        }
    }
    best
}

/// Generate evenly-spaced breaks for legends (strictly returns n breaks)
/// This is simpler than extended_breaks and guarantees exactly n breaks.
pub fn legend_breaks(domain: (f64, f64), n: usize) -> Vec<f64> {
    let (min, max) = domain;
    
    if n == 0 {
        return Vec::new();
    }
    
    if n == 1 {
        return vec![(min + max) / 2.0];
    }
    
    if n == 2 {
        return vec![min, max];
    }
    
    // Generate n evenly-spaced breaks
    (0..n)
        .map(|i| min + (max - min) * (i as f64) / ((n - 1) as f64))
        .collect()
}

/// Format axis labels using ggplot2-style logic
///
/// This implements dynamic precision based on the spacing between breaks,
/// adds thousands separators for large numbers, and uses scientific notation
/// for extreme values.
pub fn format_breaks(breaks: &[f64]) -> Vec<String> {
    if breaks.is_empty() {
        return Vec::new();
    }

    // Calculate the minimum spacing between consecutive breaks
    let min_diff = if breaks.len() > 1 {
        breaks
            .windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .filter(|&d| d > 1e-10)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0)
    } else {
        1.0
    };

    // Determine decimal places needed based on minimum difference
    // e.g., if min_diff = 0.01, we need 2 decimal places
    let decimal_places = if min_diff >= 1.0 {
        0
    } else {
        (-min_diff.log10().floor() as i32).max(0) as usize
    };

    breaks
        .iter()
        .map(|&b| format_number(b, decimal_places))
        .collect()
}

/// Format a single number with the specified decimal places
///
/// Adds thousands separators and handles special cases like very large/small numbers.
pub fn format_number(value: f64, decimal_places: usize) -> String {
    // Handle zero
    if value.abs() < 1e-10 {
        return "0".to_string();
    }

    // Use scientific notation for very large or very small numbers
    if value.abs() < 1e-4 || value.abs() >= 1e6 {
        return format!("{:.precision$e}", value, precision = decimal_places.min(2));
    }

    // Format with appropriate decimal places
    let formatted = if decimal_places == 0 {
        format!("{:.0}", value)
    } else {
        // Format with decimals, then trim trailing zeros
        let s = format!("{:.precision$}", value, precision = decimal_places);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    };

    // Add thousands separators for integers
    if decimal_places == 0 {
        add_thousands_separator(&formatted)
    } else {
        // Add thousands separators to the integer part only
        if let Some(dot_pos) = formatted.find('.') {
            let int_part = &formatted[..dot_pos];
            let dec_part = &formatted[dot_pos..];
            format!("{}{}", add_thousands_separator(int_part), dec_part)
        } else {
            formatted
        }
    }
}

/// Add thousands separators to a number string
fn add_thousands_separator(s: &str) -> String {
    let is_negative = s.starts_with('-');
    let digits = if is_negative { &s[1..] } else { s };
    
    if digits.len() <= 3 {
        return s.to_string();
    }

    let mut result = String::new();
    if is_negative {
        result.push('-');
    }

    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result
}
