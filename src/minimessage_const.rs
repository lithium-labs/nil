use core::slice;

const SCALE: i32 = 1024; // Fixed point scale (1.0 = 1024)

const fn lerp_u8(a: u8, b: u8, t: i32) -> u8 {
    let a_i32 = a as i32;
    let b_i32 = b as i32;
    // t is 0..SCALE
    // We strictly cast to ensure no overflow, though u8 fits in i32 easily.
    let res = a_i32 + ((b_i32 - a_i32) * t) / SCALE;
    if res < 0 {
        0
    } else if res > 255 {
        255
    } else {
        res as u8
    }
}

// --- Data Structures ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

// better string simply
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct ConstStr<const N: usize> {
    pub buffer: [u8; N],
    pub len: usize,
}

impl<const N: usize> AsRef<str> for ConstStr<N> {
    #[inline]
    fn as_ref(&self) -> &str {
        unsafe {
            let bytes = self.buffer.get_unchecked(..self.len);
            str::from_utf8_unchecked(bytes)
        }
    }
}
impl<const N: usize> ConstStr<N> {
    pub const fn new() -> Self {
        Self {
            buffer: [0; N],
            len: 0,
        }
    }

    pub const fn as_str(&self) -> &str {
        unsafe {
            str::from_utf8_unchecked(core::slice::from_raw_parts(self.buffer.as_ptr(), self.len))
        }
    }
    pub const fn push_str(&mut self, s: &str) {
        let bytes = s.as_bytes();
        let mut i = 0;
        while i < bytes.len() && self.len < N {
            self.buffer[self.len] = bytes[i];
            self.len += 1;
            i += 1;
        }
    }

    pub const fn push_u8n(&mut self, mut n: u8) {
        if n >= 100 {
            self.push_u8(b'0' + n / 100);
            n %= 100;
        }
        if n >= 10 {
            self.push_u8(b'0' + n / 10);
            n %= 10;
        }

        self.push_u8(b'0' + n);
    }
    pub const fn push_u8(&mut self, b: u8) {
        if self.len < N {
            self.buffer[self.len] = b;
            self.len += 1;
        }
    }

    #[inline(always)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    pub fn push_u64(&mut self, mut n: u64) {
        if n == 0 {
            self.push_u8(b'0');
            return;
        }
        let mut buf = [0u8; 20];
        let mut i = 20;

        while n > 0 {
            i -= 1;
            buf[i] = (n % 10) as u8 + b'0';
            n /= 10;
        }
        let digit_slice = &buf[i..];
        for &byte in digit_slice {
            self.push_u8(byte);
        }
    }
}

impl<const N: usize> From<&str> for ConstStr<N> {
    #[inline]
    fn from(s: &str) -> Self {
        let mut target = Self {
            buffer: [0u8; N],
            len: 0,
        };
        target.push_str(s);
        target
    }
}
impl<const N: usize> core::ops::Deref for ConstStr<N> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<const N: usize> Default for ConstStr<N> {
    #[inline]
    fn default() -> Self {
        Self {
            buffer: [0u8; N],
            len: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    None,
    Solid(Color),
    Gradient {
        start: Color,
        end: Color,
        phase: i32,
    },
    Rainbow {
        phase: i32,
        reverse: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub fill: Fill,
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub underlined: Option<bool>,
    pub strikethrough: Option<bool>,
    pub obfuscated: Option<bool>,
}

impl Style {
    pub const fn default() -> Self {
        Self {
            fill: Fill::None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
        }
    }
}

// --- Helpers ---

const fn parse_u8_hex(input: &str) -> Option<u8> {
    let bytes = input.as_bytes();
    if bytes.len() != 2 {
        return None;
    }

    let mut val = 0;
    let mut i = 0;
    while i < 2 {
        let b = bytes[i];
        let n = match b {
            b'0'..=b'9' => b - b'0',
            b'a'..=b'f' => b - b'a' + 10,
            b'A'..=b'F' => b - b'A' + 10,
            _ => return None,
        };
        val = (val << 4) | n;
        i += 1;
    }
    Some(val)
}

const fn parse_hex_color(hex: &str) -> Option<Color> {
    let mut s = hex;
    // strip #
    if let Some(rest) = strip_prefix(s, "#") {
        s = rest;
    }

    if s.len() != 6 {
        return None;
    }

    let r_str = substr(s, 0, 2);
    let g_str = substr(s, 2, 4);
    let b_str = substr(s, 4, 6);

    match (
        parse_u8_hex(r_str),
        parse_u8_hex(g_str),
        parse_u8_hex(b_str),
    ) {
        (Some(r), Some(g), Some(b)) => Some(Color::new(r, g, b)),
        _ => None,
    }
}

// Helper for string slicing in const
const fn substr(s: &str, start: usize, end: usize) -> &str {
    let b = s.as_bytes();
    if start >= b.len() || end > b.len() || start > end {
        return "";
    }
    unsafe { str::from_utf8_unchecked(slice::from_raw_parts(b.as_ptr().add(start), end - start)) }
}

const fn strip_prefix<'a>(s: &'a str, prefix: &str) -> Option<&'a str> {
    let sb = s.as_bytes();
    let pb = prefix.as_bytes();
    if sb.len() < pb.len() {
        return None;
    }

    let mut i = 0;
    while i < pb.len() {
        if sb[i] != pb[i] {
            return None;
        }
        i += 1;
    }
    Some(substr(s, pb.len(), sb.len()))
}

const fn eq_ignore_case(a: &str, b: &str) -> bool {
    let ab = a.as_bytes();
    let bb = b.as_bytes();
    if ab.len() != bb.len() {
        return false;
    }
    let mut i = 0;
    while i < ab.len() {
        let mut c1 = ab[i];
        let mut c2 = bb[i];
        if c1 >= b'A' && c1 <= b'Z' {
            c1 += 32;
        }
        if c2 >= b'A' && c2 <= b'Z' {
            c2 += 32;
        }
        if c1 != c2 {
            return false;
        }
        i += 1;
    }
    true
}

const fn color_from_name(name: &str) -> Option<Color> {
    if eq_ignore_case(name, "black") {
        return Some(Color::new(0, 0, 0));
    }
    if eq_ignore_case(name, "white") {
        return Some(Color::new(255, 255, 255));
    }
    if eq_ignore_case(name, "red") {
        return Some(Color::new(255, 85, 85));
    }
    if eq_ignore_case(name, "green") {
        return Some(Color::new(85, 255, 85));
    }
    if eq_ignore_case(name, "blue") {
        return Some(Color::new(85, 85, 255));
    }
    if eq_ignore_case(name, "cyan") {
        return Some(Color::new(85, 255, 255));
    }
    if eq_ignore_case(name, "magenta") {
        return Some(Color::new(255, 85, 255));
    }
    if eq_ignore_case(name, "gold") {
        return Some(Color::new(255, 170, 0));
    }
    if eq_ignore_case(name, "yellow") {
        return Some(Color::new(255, 255, 85));
    }
    if eq_ignore_case(name, "gray") {
        return Some(Color::new(170, 170, 170));
    }
    if eq_ignore_case(name, "dark_gray") {
        return Some(Color::new(85, 85, 85));
    }
    None
}

// Unified resolver for hex or name
const fn resolve_color(s: &str) -> Option<Color> {
    if let Some(c) = parse_hex_color(s) {
        return Some(c);
    }
    color_from_name(s)
}

// --- Math ---

const fn hsv_to_rgb(h_fixed: i32, s_fixed: i32, v_fixed: i32) -> Color {
    // H: 0..SCALE (0..1.0), S: 0..SCALE, V: 0..SCALE
    let h = h_fixed % SCALE; // Wrap hue

    let c = (v_fixed * s_fixed) / SCALE; // Chroma

    // H' = H / (SCALE / 6) = H * 6 / SCALE
    let h_prime = (h * 6) / SCALE;
    let h_rem = (h * 6) % SCALE; // Fractional part of sector

    let x = (c
        * (SCALE
            - (if (h_prime % 2) == 0 {
                SCALE - h_rem
            } else {
                h_rem
            })))
        / SCALE;

    let (r1, g1, b1) = match h_prime {
        0 => (c, x, 0),
        1 => (x, c, 0),
        2 => (0, c, x),
        3 => (0, x, c),
        4 => (x, 0, c),
        _ => (c, 0, x),
    };

    let m = v_fixed - c;

    Color::new(
        ((r1 + m) * 255 / SCALE) as u8,
        ((g1 + m) * 255 / SCALE) as u8,
        ((b1 + m) * 255 / SCALE) as u8,
    )
}

// --- Layout & rendering ---

const fn count_text_content(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut count = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2; // Skip escaped
            count += 1;
        } else if bytes[i] == b'<' {
            // Skip tag
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            i += 1;
        } else {
            count += 1;
            i += 1;
        }
    }
    count
}

#[inline(never)]
pub const fn serialize<const CAP: usize>(input: &str) -> ConstStr<CAP> {
    let mut out = ConstStr::new();
    let bytes = input.as_bytes();
    let mut idx = 0;

    // Style Stack
    let mut stack: [Style; 16] = [Style::default(); 16];
    let mut sp = 0; // Stack pointer

    // Render State
    let mut current_bold = false;
    let mut current_italic = false;
    let mut current_underlined = false;
    let mut current_strikethrough = false;
    let mut current_obfuscated = false;
    let mut current_ansi: Option<u8> = None;
    let mut current_color: Option<Color> = None;

    // Gradient Context
    let mut grad_active = false;
    let mut grad_depth = 0;
    let mut grad_char_count = 0;
    let mut grad_total_len = 1;

    while idx < bytes.len() {
        if bytes[idx] == b'\\' {
            // Escaped char
            if idx + 1 < bytes.len() {
                idx += 1;
                // Render char
                let c = bytes[idx];

                // --- RENDER LOGIC START ---
                let style = stack[sp];
                let mut fg = None;

                match style.fill {
                    Fill::Solid(c) => fg = Some(c),
                    Fill::Gradient { start, end, phase } => {
                        if grad_active {
                            let t = (grad_char_count * SCALE) / grad_total_len;
                            let t_phased = (t + phase) % SCALE;
                            fg = Some(Color::new(
                                lerp_u8(start.r, end.r, t_phased),
                                lerp_u8(start.g, end.g, t_phased),
                                lerp_u8(start.b, end.b, t_phased),
                            ));
                        }
                    }
                    Fill::Rainbow { phase, reverse } => {
                        if grad_active {
                            let step = (grad_char_count * SCALE) / grad_total_len;
                            let mut hue = step + phase;
                            if reverse {
                                hue = SCALE - hue;
                            }
                            fg = Some(hsv_to_rgb(hue, SCALE, SCALE));
                        }
                    }
                    Fill::None => {}
                }

                let mut reset = false;
                if current_bold && style.bold != Some(true) {
                    reset = true;
                }
                if current_italic && style.italic != Some(true) {
                    reset = true;
                }
                if current_underlined && style.underlined != Some(true) {
                    reset = true;
                }
                if current_strikethrough && style.strikethrough != Some(true) {
                    reset = true;
                }
                if current_obfuscated && style.obfuscated != Some(true) {
                    reset = true;
                }
                if current_color.is_some() && fg.is_none() {
                    reset = true;
                }
                if current_ansi.is_some() {
                    if let Fill::None = style.fill {
                        reset = true;
                    }
                }

                if reset {
                    out.push_str("\x1b[0m");
                    current_bold = false;
                    current_italic = false;
                    current_underlined = false;
                    current_strikethrough = false;
                    current_obfuscated = false;
                    current_ansi = None;
                    current_color = None;
                }

                if style.bold == Some(true) && !current_bold {
                    out.push_str("\x1b[1m");
                    current_bold = true;
                }
                if style.italic == Some(true) && !current_italic {
                    out.push_str("\x1b[3m");
                    current_italic = true;
                }
                if style.underlined == Some(true) && !current_underlined {
                    out.push_str("\x1b[4m");
                    current_underlined = true;
                }

                if let Some(c) = fg {
                    let mut update = false;
                    if let Some(curr) = current_color {
                        if curr.r != c.r || curr.g != c.g || curr.b != c.b {
                            update = true;
                        }
                    } else {
                        update = true;
                    }

                    if update {
                        out.push_str("\x1b[38;2;");
                        out.push_u8n(c.r);
                        out.push_str(";");
                        out.push_u8n(c.g);
                        out.push_str(";");
                        out.push_u8n(c.b);
                        out.push_str("m");
                        current_color = Some(c);
                    }
                }

                out.push_u8(c);
                if grad_active {
                    grad_char_count += 1;
                }
                // --- RENDER LOGIC END ---
            }
            idx += 1;
        } else if bytes[idx] == b'<' {
            // Parse Tag
            let start = idx + 1;
            let mut end = start;
            while end < bytes.len() && bytes[end] != b'>' {
                end += 1;
            }

            if end < bytes.len() {
                let content = substr(input, start, end);
                let is_close = if !content.is_empty() && content.as_bytes()[0] == b'/' {
                    true
                } else {
                    false
                };

                if is_close {
                    // Pop
                    if sp > 0 {
                        sp -= 1;
                        if grad_active && sp < grad_depth {
                            grad_active = false;
                        }
                    }
                } else {
                    // Push
                    if sp < 15 {
                        let mut new_style = stack[sp];

                        let b_cont = content.as_bytes();
                        let mut colon = b_cont.len();
                        let mut k = 0;
                        while k < b_cont.len() {
                            if b_cont[k] == b':' {
                                colon = k;
                                break;
                            }
                            k += 1;
                        }

                        let name = substr(content, 0, colon);
                        let args_str = if colon < b_cont.len() {
                            substr(content, colon + 1, b_cont.len())
                        } else {
                            ""
                        };

                        // Apply Styles
                        if eq_ignore_case(name, "b") || eq_ignore_case(name, "bold") {
                            new_style.bold = Some(true);
                        } else if eq_ignore_case(name, "i") || eq_ignore_case(name, "italic") {
                            new_style.italic = Some(true);
                        } else if eq_ignore_case(name, "u") || eq_ignore_case(name, "underlined") {
                            new_style.underlined = Some(true);
                        } else if eq_ignore_case(name, "rainbow") {
                            new_style.fill = Fill::Rainbow {
                                phase: 0,
                                reverse: false,
                            };
                            if !grad_active {
                                grad_active = true;
                                grad_depth = sp + 1;
                                grad_char_count = 0;
                                let remaining = substr(input, end + 1, input.len());
                                grad_total_len = count_text_content(remaining) as i32;
                                if grad_total_len == 0 {
                                    grad_total_len = 1;
                                }
                            }
                        } else if eq_ignore_case(name, "gradient") {
                            let mut c1 = Color::new(255, 255, 255);
                            let mut c2 = Color::new(0, 0, 0);

                            // Parse args "color1:color2"
                            if !args_str.is_empty() {
                                let b_args = args_str.as_bytes();
                                let mut colon_idx = 0;
                                let mut found_colon = false;
                                while colon_idx < b_args.len() {
                                    if b_args[colon_idx] == b':' {
                                        found_colon = true;
                                        break;
                                    }
                                    colon_idx += 1;
                                }

                                if found_colon {
                                    let s1 = substr(args_str, 0, colon_idx);
                                    let s2 = substr(args_str, colon_idx + 1, b_args.len());
                                    if let Some(c) = resolve_color(s1) {
                                        c1 = c;
                                    }
                                    if let Some(c) = resolve_color(s2) {
                                        c2 = c;
                                    }
                                } else {
                                    // Single color fallback
                                    if let Some(c) = resolve_color(args_str) {
                                        c1 = c;
                                        c2 = c;
                                    }
                                }
                            }

                            new_style.fill = Fill::Gradient {
                                start: c1,
                                end: c2,
                                phase: 0,
                            };

                            if !grad_active {
                                grad_active = true;
                                grad_depth = sp + 1;
                                grad_char_count = 0;
                                let remaining = substr(input, end + 1, input.len());
                                grad_total_len = count_text_content(remaining) as i32;
                                if grad_total_len == 0 {
                                    grad_total_len = 1;
                                }
                            }
                        } else if let Some(c) = resolve_color(name) {
                            new_style.fill = Fill::Solid(c);
                        }

                        sp += 1;
                        stack[sp] = new_style;
                    }
                }
                idx = end + 1;
            } else {
                idx += 1;
            }
        } else {
            let c = bytes[idx];

            // --- RENDER LOGIC START (Normal Char) ---
            let style = stack[sp];
            let mut fg = None;

            match style.fill {
                Fill::Solid(c) => fg = Some(c),
                Fill::Gradient { start, end, phase } => {
                    if grad_active {
                        let t = (grad_char_count as i32 * SCALE) / grad_total_len as i32;
                        let t_phased = (t + phase) % SCALE;
                        fg = Some(Color::new(
                            lerp_u8(start.r, end.r, t_phased),
                            lerp_u8(start.g, end.g, t_phased),
                            lerp_u8(start.b, end.b, t_phased),
                        ));
                    }
                }
                Fill::Rainbow { phase, reverse } => {
                    if grad_active {
                        let step = (grad_char_count as i32 * SCALE) / grad_total_len as i32;
                        let mut hue = step + phase;
                        if reverse {
                            hue = SCALE - hue;
                        }
                        fg = Some(hsv_to_rgb(hue, SCALE, SCALE));
                    }
                }
                Fill::None => {}
            }

            let mut reset = false;
            if current_bold && style.bold != Some(true) {
                reset = true;
            }
            if current_italic && style.italic != Some(true) {
                reset = true;
            }
            if current_underlined && style.underlined != Some(true) {
                reset = true;
            }
            if current_color.is_some() && fg.is_none() {
                reset = true;
            }

            if reset {
                out.push_str("\x1b[0m");
                current_bold = false;
                current_italic = false;
                current_underlined = false;
                current_color = None;
            }

            if style.bold == Some(true) && !current_bold {
                out.push_str("\x1b[1m");
                current_bold = true;
            }
            if style.italic == Some(true) && !current_italic {
                out.push_str("\x1b[3m");
                current_italic = true;
            }
            if style.underlined == Some(true) && !current_underlined {
                out.push_str("\x1b[4m");
                current_underlined = true;
            }

            if let Some(c) = fg {
                let mut update = true;
                if let Some(curr) = current_color {
                    if curr.r == c.r && curr.g == c.g && curr.b == c.b {
                        update = false;
                    }
                }

                if update {
                    out.push_str("\x1b[38;2;");
                    out.push_u8n(c.r);
                    out.push_str(";");
                    out.push_u8n(c.g);
                    out.push_str(";");
                    out.push_u8n(c.b);
                    out.push_str("m");
                    current_color = Some(c);
                }
            }

            out.push_u8(c);
            if grad_active {
                grad_char_count += 1;
            }
            // --- RENDER LOGIC END ---

            idx += 1;
        }
    }

    out.push_str("\x1b[0m");
    out
}
