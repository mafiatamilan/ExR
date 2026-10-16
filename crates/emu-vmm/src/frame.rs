//! PPM frame helpers for the M1 boot proof.
//!
//! QMP `screendump` writes P6 PPM files. These pure functions answer the two
//! M1 questions without any image library: *is the frame non-blank?* and
//! *did it change between captures?*

use std::path::Path;

/// Minimal parsed P6 PPM (as written by QEMU screendump).
#[derive(Debug)]
pub struct Ppm {
    /// RGB bytes, row-major.
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// Parse a binary P6 PPM file. Rejects anything else (including P3 ASCII).
pub fn parse_ppm(path: &Path) -> Result<Ppm, String> {
    let data = std::fs::read(path).map_err(|e| format!("read {}: {e}", path.display()))?;
    // Header: "P6" <ws> <w> <ws> <h> <ws> <max> <single-ws>, comments (#...) allowed.
    let mut pos: usize;
    let mut tokens: Vec<Vec<u8>> = Vec::new();
    // magic
    if data.len() < 2 || &data[0..2] != b"P6" {
        return Err("not a P6 PPM".into());
    }
    pos = 2;
    while tokens.len() < 3 && pos < data.len() {
        // skip whitespace
        while pos < data.len() && data[pos].is_ascii_whitespace() {
            pos += 1;
        }
        // skip comments
        if pos < data.len() && data[pos] == b'#' {
            while pos < data.len() && data[pos] != b'\n' {
                pos += 1;
            }
            continue;
        }
        let start = pos;
        while pos < data.len() && !data[pos].is_ascii_whitespace() {
            pos += 1;
        }
        if start < pos {
            tokens.push(data[start..pos].to_vec());
        }
    }
    if tokens.len() != 3 {
        return Err("malformed PPM header".into());
    }
    let width: u32 = std::str::from_utf8(&tokens[0])
        .map_err(|_| "bad width".to_string())?
        .parse()
        .map_err(|_| "bad width".to_string())?;
    let height: u32 = std::str::from_utf8(&tokens[1])
        .map_err(|_| "bad height".to_string())?
        .parse()
        .map_err(|_| "bad height".to_string())?;
    let maxval: u32 = std::str::from_utf8(&tokens[2])
        .map_err(|_| "bad maxval".to_string())?
        .parse()
        .map_err(|_| "bad maxval".to_string())?;
    if maxval != 255 {
        return Err(format!("unsupported maxval {maxval} (need 255)"));
    }
    // exactly one whitespace byte follows maxval
    if pos >= data.len() || !data[pos].is_ascii_whitespace() {
        return Err("missing header terminator".into());
    }
    pos += 1;
    let expect = width as usize * height as usize * 3;
    if data.len() - pos < expect {
        return Err(format!(
            "truncated pixels: have {}, need {expect}",
            data.len() - pos
        ));
    }
    Ok(Ppm {
        pixels: data[pos..pos + expect].to_vec(),
        width,
        height,
    })
}

/// Fraction of pixels that are not pure black (0.0–1.0).
pub fn nonblack_fraction(ppm: &Ppm) -> f64 {
    if ppm.pixels.is_empty() {
        return 0.0;
    }
    let lit = ppm
        .pixels
        .chunks_exact(3)
        .filter(|px| px != &[0, 0, 0])
        .count();
    lit as f64 / (ppm.pixels.len() / 3) as f64
}

/// A frame counts as *non-blank* when at least 0.1% of pixels are lit.
/// (SeaBIOS/VGA-text frames clear this easily; a dead/black capture does not.)
pub fn is_nonblank(ppm: &Ppm) -> bool {
    nonblack_fraction(ppm) >= 0.001
}

/// Fraction of pixels differing between two same-size frames (0.0–1.0).
/// Returns `None` when dimensions differ.
pub fn changed_fraction(a: &Ppm, b: &Ppm) -> Option<f64> {
    if a.width != b.width || a.height != b.height || a.pixels.len() != b.pixels.len() {
        return None;
    }
    let total = a.pixels.len() / 3;
    if total == 0 {
        return Some(0.0);
    }
    let diff = a
        .pixels
        .chunks_exact(3)
        .zip(b.pixels.chunks_exact(3))
        .filter(|(x, y)| x != y)
        .count();
    Some(diff as f64 / total as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ppm_of(w: u32, h: u32, fill: [u8; 3]) -> Ppm {
        Ppm {
            pixels: std::iter::repeat_n(fill, (w * h) as usize)
                .flatten()
                .collect(),
            width: w,
            height: h,
        }
    }

    #[test]
    fn blank_is_blank() {
        let black = ppm_of(64, 64, [0, 0, 0]);
        assert!(!is_nonblank(&black));
        assert_eq!(nonblack_fraction(&black), 0.0);
    }

    #[test]
    fn tiny_text_counts_as_content() {
        let mut f = ppm_of(64, 64, [0, 0, 0]);
        // 5 lit pixels out of 4096 = 0.12% — like a small cursor/boot glyph
        for i in 0..5 {
            f.pixels[i * 3..i * 3 + 3].copy_from_slice(&[200, 200, 200]);
        }
        assert!(is_nonblank(&f));
    }

    #[test]
    fn identical_frames_show_no_change() {
        let a = ppm_of(32, 32, [10, 20, 30]);
        let b = ppm_of(32, 32, [10, 20, 30]);
        assert_eq!(changed_fraction(&a, &b), Some(0.0));
    }

    #[test]
    fn changed_frames_detected() {
        let a = ppm_of(10, 10, [0, 0, 0]);
        let mut b = ppm_of(10, 10, [0, 0, 0]);
        for i in 0..50 {
            b.pixels[i * 3] = 255;
        }
        assert_eq!(changed_fraction(&a, &b), Some(0.5));
    }

    #[test]
    fn size_mismatch_is_none() {
        let a = ppm_of(8, 8, [1, 1, 1]);
        let b = ppm_of(8, 9, [1, 1, 1]);
        assert_eq!(changed_fraction(&a, &b), None);
    }

    #[test]
    fn parses_real_p6_file() {
        let dir = std::env::temp_dir();
        let p = dir.join("exr-ppm-test.ppm");
        let mut bytes = b"P6\n#Lifetime comment\n4 2\n255\n".to_vec();
        bytes.extend([
            255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        std::fs::write(&p, &bytes).unwrap();
        let ppm = parse_ppm(&p).unwrap();
        assert_eq!((ppm.width, ppm.height), (4, 2));
        assert!(is_nonblank(&ppm));
        let _ = std::fs::remove_file(&p);
    }

    #[test]
    fn rejects_non_ppm() {
        let dir = std::env::temp_dir();
        let p = dir.join("exr-ppm-bad.ppm");
        std::fs::write(&p, b"P3\n2 2\n255\n0 0 0 0 0 0 0 0 0 0 0 0\n").unwrap();
        assert!(parse_ppm(&p).is_err());
        let _ = std::fs::remove_file(&p);
    }
}
