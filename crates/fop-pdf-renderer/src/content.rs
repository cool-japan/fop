//! PDF content stream interpreter
//!
//! Tokenizes a PDF content stream and interprets operators,
//! producing drawing commands for the rasterizer.

use crate::error::Result;
use crate::font::LoadedFont;
use crate::graphics::{Color, CurrentPath, GraphicsStateStack, Matrix};
use crate::image::DecodedImage;
use crate::parser::{PdfDictionary, PdfDocument};
use crate::text::{decode_string_to_cids, PositionedGlyph, TextState};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// A content stream token
#[derive(Debug, Clone)]
enum Token {
    Number(f64),
    Name(String),
    StringBytes(Vec<u8>),
    ArrayOpen,
    ArrayClose,
    Operator(String),
}

struct Tokenizer<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn next_token(&mut self) -> Option<Token> {
        self.skip_whitespace_and_comments();
        if self.pos >= self.data.len() {
            return None;
        }

        match self.data[self.pos] {
            b'[' => {
                self.pos += 1;
                Some(Token::ArrayOpen)
            }
            b']' => {
                self.pos += 1;
                Some(Token::ArrayClose)
            }
            b'(' => Some(Token::StringBytes(self.read_literal_string())),
            b'<' => {
                if self.data.get(self.pos + 1) == Some(&b'<') {
                    // inline dictionary — skip
                    self.skip_dict();
                    self.next_token()
                } else {
                    Some(Token::StringBytes(self.read_hex_string()))
                }
            }
            b'/' => {
                self.pos += 1;
                Some(Token::Name(self.read_name()))
            }
            c if c == b'-' || c == b'+' || c.is_ascii_digit() || c == b'.' => {
                Some(Token::Number(self.read_number()))
            }
            _ => Some(Token::Operator(self.read_operator())),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' | b'\x0C' => self.pos += 1,
                b'%' => {
                    while self.pos < self.data.len() && self.data[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn read_literal_string(&mut self) -> Vec<u8> {
        self.pos += 1; // skip '('
        let mut result = Vec::new();
        let mut depth = 1i32;
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b'\\' => {
                    self.pos += 1;
                    if self.pos < self.data.len() {
                        let c = match self.data[self.pos] {
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            b'b' => 0x08,
                            b'f' => 0x0C,
                            b'(' => b'(',
                            b')' => b')',
                            b'\\' => b'\\',
                            c => c,
                        };
                        result.push(c);
                        self.pos += 1;
                    }
                }
                b'(' => {
                    depth += 1;
                    result.push(b'(');
                    self.pos += 1;
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        self.pos += 1;
                        break;
                    }
                    result.push(b')');
                    self.pos += 1;
                }
                c => {
                    result.push(c);
                    self.pos += 1;
                }
            }
        }
        result
    }

    fn read_hex_string(&mut self) -> Vec<u8> {
        self.pos += 1; // skip '<'
        let mut result = Vec::new();
        let mut nibble: Option<u8> = None;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'>' {
                self.pos += 1;
                break;
            }
            if c.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }
            let digit = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => {
                    self.pos += 1;
                    continue;
                }
            };
            match nibble {
                None => {
                    nibble = Some(digit << 4);
                }
                Some(hi) => {
                    result.push(hi | digit);
                    nibble = None;
                }
            }
            self.pos += 1;
        }
        if let Some(hi) = nibble {
            result.push(hi);
        }
        result
    }

    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c.is_ascii_whitespace() || b"/<>[](){}".contains(&c) {
                break;
            }
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.data[start..self.pos]).into_owned()
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        if self.pos < self.data.len()
            && (self.data[self.pos] == b'-' || self.data[self.pos] == b'+')
        {
            self.pos += 1;
        }
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'.' {
            self.pos += 1;
            while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }
        let s = std::str::from_utf8(&self.data[start..self.pos]).unwrap_or("0");
        s.parse::<f64>().unwrap_or(0.0)
    }

    fn read_operator(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c.is_ascii_whitespace() || b"/<>[](){}".contains(&c) {
                break;
            }
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.data[start..self.pos]).into_owned()
    }

    fn skip_dict(&mut self) {
        self.pos += 2; // skip "<<"
        let mut depth = 1;
        while self.pos < self.data.len() && depth > 0 {
            match self.data[self.pos] {
                b'<' if self.data.get(self.pos + 1) == Some(&b'<') => {
                    depth += 1;
                    self.pos += 2;
                }
                b'>' if self.data.get(self.pos + 1) == Some(&b'>') => {
                    depth -= 1;
                    self.pos += 2;
                }
                _ => self.pos += 1,
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Operand stack helpers
// ---------------------------------------------------------------------------

fn pop_f(stack: &mut Vec<Token>) -> f32 {
    match stack.pop() {
        Some(Token::Number(n)) => n as f32,
        _ => 0.0,
    }
}

fn pop_i(stack: &mut Vec<Token>) -> i64 {
    match stack.pop() {
        Some(Token::Number(n)) => n as i64,
        _ => 0,
    }
}

fn pop_str(stack: &mut Vec<Token>) -> Vec<u8> {
    match stack.pop() {
        Some(Token::StringBytes(b)) => b,
        _ => Vec::new(),
    }
}

fn pop_name(stack: &mut Vec<Token>) -> String {
    match stack.pop() {
        Some(Token::Name(n)) => n,
        Some(Token::Operator(n)) => n,
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// DrawCommand — output of the interpreter
// ---------------------------------------------------------------------------

/// A drawing command produced by interpreting the content stream
#[derive(Debug)]
pub enum DrawCommand {
    /// Fill a path with the given color
    FillPath {
        path: tiny_skia::Path,
        color: Color,
        fill_rule: FillRule,
    },
    /// Stroke a path
    StrokePath {
        path: tiny_skia::Path,
        color: Color,
        width: f32,
    },
    /// Render a glyph
    DrawGlyph(PositionedGlyph),
    /// Render an image
    DrawImage {
        image: DecodedImage,
        transform: Matrix,
    },
}

#[derive(Debug, Clone, Copy)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

// ---------------------------------------------------------------------------
// Interpreter
// ---------------------------------------------------------------------------

/// Interprets a PDF content stream, producing draw commands
pub struct ContentInterpreter<'a> {
    doc: &'a PdfDocument,
    resources: &'a PdfDictionary,
    gs_stack: GraphicsStateStack,
    text_state: TextState,
    path: CurrentPath,
    /// Loaded fonts cache: resource name → LoadedFont
    font_cache: HashMap<String, LoadedFont>,
    /// Page height for Y-flip (PDF Y=0 is bottom, screen Y=0 is top)
    pub page_height: f32,
}

impl<'a> ContentInterpreter<'a> {
    pub fn new(doc: &'a PdfDocument, resources: &'a PdfDictionary, page_height: f32) -> Self {
        Self {
            doc,
            resources,
            gs_stack: GraphicsStateStack::default(),
            text_state: TextState::default(),
            path: CurrentPath::default(),
            font_cache: HashMap::new(),
            page_height,
        }
    }

    /// Interpret the content stream, returning draw commands
    pub fn interpret(&mut self, content: &[u8]) -> Result<Vec<DrawCommand>> {
        let mut commands = Vec::new();
        let mut tokenizer = Tokenizer::new(content);
        let mut operand_stack: Vec<Token> = Vec::new();
        let mut array_mode = false;
        let mut array_buf: Vec<Token> = Vec::new();

        while let Some(token) = tokenizer.next_token() {
            match &token {
                Token::ArrayOpen => {
                    array_mode = true;
                    array_buf.clear();
                }
                Token::ArrayClose => {
                    array_mode = false;
                    let arr = std::mem::take(&mut array_buf);
                    operand_stack.push(Token::StringBytes(
                        // Store TJ array items as combined bytes with spacing hints
                        arr.into_iter()
                            .flat_map(|t| match t {
                                Token::StringBytes(b) => b,
                                Token::Number(_) => Vec::new(), // kerning adjustments handled separately
                                _ => Vec::new(),
                            })
                            .collect(),
                    ));
                    // Re-interpret as TJ after operator comes
                }
                Token::Operator(op) => {
                    self.execute_operator(op.as_str(), &mut operand_stack, &mut commands)?;
                    operand_stack.clear();
                }
                _ => {
                    if array_mode {
                        array_buf.push(token);
                    } else {
                        operand_stack.push(token);
                    }
                }
            }
        }

        Ok(commands)
    }

    fn execute_operator(
        &mut self,
        op: &str,
        stack: &mut Vec<Token>,
        commands: &mut Vec<DrawCommand>,
    ) -> Result<()> {
        match op {
            // --------------- Graphics state ---------------
            "q" => self.gs_stack.push(),
            "Q" => self.gs_stack.pop(),
            "cm" => {
                // Stack has: a b c d e f (pushed left to right, so f is on top)
                let f = pop_f(stack);
                let e = pop_f(stack);
                let d = pop_f(stack);
                let c = pop_f(stack);
                let b = pop_f(stack);
                let a = pop_f(stack);
                self.gs_stack.concat_ctm(Matrix { a, b, c, d, e, f });
            }
            "w" => {
                let w = pop_f(stack);
                self.gs_stack.current_mut().line_width = w;
            }
            "J" => {
                let _ = pop_i(stack); /* line cap */
            }
            "j" => {
                let _ = pop_i(stack); /* line join */
            }
            "M" => {
                let _ = pop_f(stack); /* miter limit */
            }
            "d" => {
                stack.clear(); /* dash pattern — skip */
            }
            "ri" => {
                let _ = pop_name(stack); /* rendering intent */
            }
            "i" => {
                let _ = pop_f(stack); /* flatness */
            }
            "gs" => {
                let _ = pop_name(stack); /* external graphics state — skip */
            }

            // --------------- Color ---------------
            "g" => {
                let g = pop_f(stack);
                self.gs_stack.current_mut().fill_color = Color::gray(g);
            }
            "G" => {
                let g = pop_f(stack);
                self.gs_stack.current_mut().stroke_color = Color::gray(g);
            }
            "rg" => {
                let b = pop_f(stack);
                let g = pop_f(stack);
                let r = pop_f(stack);
                self.gs_stack.current_mut().fill_color = Color::rgb(r, g, b);
            }
            "RG" => {
                let b = pop_f(stack);
                let g = pop_f(stack);
                let r = pop_f(stack);
                self.gs_stack.current_mut().stroke_color = Color::rgb(r, g, b);
            }
            "k" => {
                let k = pop_f(stack);
                let y = pop_f(stack);
                let m = pop_f(stack);
                let c = pop_f(stack);
                let r = (1.0 - c) * (1.0 - k);
                let g = (1.0 - m) * (1.0 - k);
                let b = (1.0 - y) * (1.0 - k);
                self.gs_stack.current_mut().fill_color = Color::rgb(r, g, b);
            }
            "K" => {
                let k = pop_f(stack);
                let y = pop_f(stack);
                let m = pop_f(stack);
                let c = pop_f(stack);
                let r = (1.0 - c) * (1.0 - k);
                let g = (1.0 - m) * (1.0 - k);
                let b = (1.0 - y) * (1.0 - k);
                self.gs_stack.current_mut().stroke_color = Color::rgb(r, g, b);
            }
            "cs" | "CS" => {
                let _ = pop_name(stack);
            }
            "sc" | "SC" | "scn" | "SCN" => {
                stack.clear();
            }

            // --------------- Path construction ---------------
            "m" => {
                let y = pop_f(stack);
                let x = pop_f(stack);
                let (sx, sy) = self.to_screen(x, y);
                self.path.move_to(sx, sy);
            }
            "l" => {
                let y = pop_f(stack);
                let x = pop_f(stack);
                let (sx, sy) = self.to_screen(x, y);
                self.path.line_to(sx, sy);
            }
            "c" => {
                let y3 = pop_f(stack);
                let x3 = pop_f(stack);
                let y2 = pop_f(stack);
                let x2 = pop_f(stack);
                let y1 = pop_f(stack);
                let x1 = pop_f(stack);
                let (sx1, sy1) = self.to_screen(x1, y1);
                let (sx2, sy2) = self.to_screen(x2, y2);
                let (sx3, sy3) = self.to_screen(x3, y3);
                self.path.curve_to(sx1, sy1, sx2, sy2, sx3, sy3);
            }
            "v" => {
                // curveto with first control = current point
                let y3 = pop_f(stack);
                let x3 = pop_f(stack);
                let y2 = pop_f(stack);
                let x2 = pop_f(stack);
                let cx = self.path.current_x;
                let cy = self.path.current_y;
                let (sx2, sy2) = self.to_screen(x2, y2);
                let (sx3, sy3) = self.to_screen(x3, y3);
                self.path.curve_to(cx, cy, sx2, sy2, sx3, sy3);
            }
            "y" => {
                // curveto with last control = end point
                let y3 = pop_f(stack);
                let x3 = pop_f(stack);
                let y1 = pop_f(stack);
                let x1 = pop_f(stack);
                let (sx1, sy1) = self.to_screen(x1, y1);
                let (sx3, sy3) = self.to_screen(x3, y3);
                self.path.curve_to(sx1, sy1, sx3, sy3, sx3, sy3);
            }
            "h" => self.path.close(),
            "re" => {
                let h = pop_f(stack);
                let w = pop_f(stack);
                let y = pop_f(stack);
                let x = pop_f(stack);
                // Convert rect corners
                let (sx, sy) = self.to_screen(x, y);
                let (sx2, sy2) = self.to_screen(x + w, y + h);
                let rx = sx.min(sx2);
                let ry = sy.min(sy2);
                let rw = (sx - sx2).abs();
                let rh = (sy - sy2).abs();
                self.path.rect(rx, ry, rw, rh);
            }

            // --------------- Path painting ---------------
            "S" => {
                // Stroke
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::StrokePath {
                        path,
                        color: state.stroke_color,
                        width: state.line_width,
                    });
                }
                self.path.clear();
            }
            "s" => {
                // Close and stroke
                self.path.close();
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::StrokePath {
                        path,
                        color: state.stroke_color,
                        width: state.line_width,
                    });
                }
                self.path.clear();
            }
            "f" | "F" => {
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::FillPath {
                        path,
                        color: state.fill_color,
                        fill_rule: FillRule::NonZero,
                    });
                }
                self.path.clear();
            }
            "f*" => {
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::FillPath {
                        path,
                        color: state.fill_color,
                        fill_rule: FillRule::EvenOdd,
                    });
                }
                self.path.clear();
            }
            "B" => {
                // Fill and stroke
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::FillPath {
                        path: path.clone(),
                        color: state.fill_color,
                        fill_rule: FillRule::NonZero,
                    });
                    commands.push(DrawCommand::StrokePath {
                        path,
                        color: state.stroke_color,
                        width: state.line_width,
                    });
                }
                self.path.clear();
            }
            "B*" => {
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::FillPath {
                        path: path.clone(),
                        color: state.fill_color,
                        fill_rule: FillRule::EvenOdd,
                    });
                    commands.push(DrawCommand::StrokePath {
                        path,
                        color: state.stroke_color,
                        width: state.line_width,
                    });
                }
                self.path.clear();
            }
            "b" => {
                self.path.close();
                if let Some(path) = self.build_path() {
                    let state = self.gs_stack.current();
                    commands.push(DrawCommand::FillPath {
                        path: path.clone(),
                        color: state.fill_color,
                        fill_rule: FillRule::NonZero,
                    });
                    commands.push(DrawCommand::StrokePath {
                        path,
                        color: state.stroke_color,
                        width: state.line_width,
                    });
                }
                self.path.clear();
            }
            "n" => {
                // End path without painting
                self.path.clear();
            }
            "W" | "W*" => {
                // Clip — skip for now
                self.path.clear();
            }

            // --------------- Text ---------------
            "BT" => {
                self.text_state = TextState::default();
            }
            "ET" => {}
            "Tf" => {
                let size = pop_f(stack);
                let name = pop_name(stack);
                self.text_state.font_name = name;
                self.text_state.font_size = size;
                // Load font into cache if needed
                self.ensure_font_loaded();
            }
            "Tc" => {
                self.text_state.char_spacing = pop_f(stack);
            }
            "Tw" => {
                self.text_state.word_spacing = pop_f(stack);
            }
            "Tz" => {
                self.text_state.horiz_scale = pop_f(stack);
            }
            "TL" => {
                self.text_state.leading = pop_f(stack);
            }
            "Ts" => {
                self.text_state.text_rise = pop_f(stack);
            }
            "Tr" => {
                self.text_state.rendering_mode = pop_i(stack) as u8;
            }
            "Td" => {
                let ty = pop_f(stack);
                let tx = pop_f(stack);
                self.text_state.td(tx, ty);
            }
            "TD" => {
                let ty = pop_f(stack);
                let tx = pop_f(stack);
                self.text_state.capital_td(tx, ty);
            }
            "Tm" => {
                let f = pop_f(stack);
                let e = pop_f(stack);
                let d = pop_f(stack);
                let c = pop_f(stack);
                let b = pop_f(stack);
                let a = pop_f(stack);
                self.text_state.tm(a, b, c, d, e, f);
            }
            "T*" => {
                self.text_state.t_star();
            }
            "Tj" => {
                let bytes = pop_str(stack);
                self.show_string(&bytes, commands);
            }
            "TJ" => {
                let bytes = pop_str(stack);
                self.show_string(&bytes, commands);
            }
            "'" => {
                // T* then Tj
                self.text_state.t_star();
                let bytes = pop_str(stack);
                self.show_string(&bytes, commands);
            }
            "\"" => {
                let bytes = pop_str(stack);
                let _ = pop_f(stack); // word spacing
                let _ = pop_f(stack); // char spacing
                self.text_state.t_star();
                self.show_string(&bytes, commands);
            }

            // --------------- XObject ---------------
            "Do" => {
                let name = pop_name(stack);
                self.draw_xobject(&name, commands)?;
            }

            // --------------- Inline image ---------------
            "BI" => { /* handled inline - skip */ }
            "EI" => {}

            // --------------- Misc ---------------
            "sh" => {
                let _ = pop_name(stack);
            } // shading — skip
            "BX" | "EX" => {}
            "BMC" | "BDC" | "EMC" => {
                stack.clear();
            }
            "MP" | "DP" => {
                stack.clear();
            }

            _ => {
                // Unknown operator
                log::trace!("Unknown PDF operator: {}", op);
                stack.clear();
            }
        }
        Ok(())
    }

    fn to_screen(&self, x: f32, y: f32) -> (f32, f32) {
        let ctm = &self.gs_stack.current().ctm;
        let (tx, ty) = ctm.transform_point(x, y);
        // Flip Y: PDF Y=0 is bottom, screen Y=0 is top
        (tx, self.page_height - ty)
    }

    fn build_path(&self) -> Option<tiny_skia::Path> {
        // Build path directly from segments (already in screen space from to_screen)
        let mut pb = tiny_skia::PathBuilder::new();
        let mut has_content = false;
        for seg in &self.path.segments {
            use crate::graphics::PathSegment;
            match *seg {
                PathSegment::MoveTo(x, y) => {
                    pb.move_to(x, y);
                    has_content = true;
                }
                PathSegment::LineTo(x, y) => {
                    pb.line_to(x, y);
                    has_content = true;
                }
                PathSegment::CurveTo(x1, y1, x2, y2, x3, y3) => {
                    pb.cubic_to(x1, y1, x2, y2, x3, y3);
                    has_content = true;
                }
                PathSegment::Close => {
                    pb.close();
                }
                PathSegment::Rect(x, y, w, h) => {
                    if let Some(r) = tiny_skia::Rect::from_xywh(x, y, w, h) {
                        pb.push_rect(r);
                        has_content = true;
                    }
                }
            }
        }
        if has_content {
            pb.finish()
        } else {
            None
        }
    }

    fn ensure_font_loaded(&mut self) {
        let name = self.text_state.font_name.clone();
        if self.font_cache.contains_key(&name) {
            return;
        }
        if let Some(font_dict) = self.doc.get_font(self.resources, &name) {
            let loaded = LoadedFont::load(self.doc, &font_dict);
            self.font_cache.insert(name, loaded);
        }
    }

    fn show_string(&mut self, bytes: &[u8], commands: &mut Vec<DrawCommand>) {
        let font_name = self.text_state.font_name.clone();
        let is_composite = self
            .font_cache
            .get(&font_name)
            .map(|f| f.subtype == "Type0")
            .unwrap_or(false);

        let cids = decode_string_to_cids(bytes, is_composite);
        let ctm = self.gs_stack.current().ctm;
        let fill_color = self.gs_stack.current().fill_color;

        for cid in cids {
            // Get current text position in user space
            let tm = &self.text_state.text_matrix;
            let (ux, uy) = (tm.e, tm.f);
            // Apply CTM
            let (px, py) = ctm.transform_point(ux, uy);
            // Flip Y
            let sy = self.page_height - py;

            // Font size in user space
            let font_size = self.text_state.font_size * ctm.a.abs();

            // Get character
            let character = self
                .font_cache
                .get(&font_name)
                .and_then(|f| f.cid_to_char(cid));

            // Get advance width
            let advance_units = self
                .font_cache
                .get(&font_name)
                .map(|f| f.advance_width(cid))
                .unwrap_or(1000.0);

            let is_space = character == Some(' ');

            commands.push(DrawCommand::DrawGlyph(PositionedGlyph {
                x: px,
                y: sy,
                font_size,
                character,
                cid,
                font_name: font_name.clone(),
                advance: (advance_units / 1000.0) * self.text_state.font_size,
                color: fill_color,
            }));

            // Advance text matrix
            self.text_state.advance(advance_units, is_space);
        }
    }

    fn draw_xobject(&mut self, name: &str, commands: &mut Vec<DrawCommand>) -> Result<()> {
        if let Some((dict, data)) = self.doc.get_xobject(self.resources, name) {
            let subtype = dict.get_name("Subtype").unwrap_or("");
            if subtype == "Image" {
                match crate::image::DecodedImage::decode(&dict, &data) {
                    Ok(img) => {
                        let ctm = self.gs_stack.current().ctm;
                        commands.push(DrawCommand::DrawImage {
                            image: img,
                            transform: ctm,
                        });
                    }
                    Err(e) => {
                        log::warn!("Image decode error for {}: {}", name, e);
                    }
                }
            }
            // Form XObjects (subtype=Form) would recurse into content stream
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Tokenizer helpers: test via Tokenizer directly (accessible from tests mod)
    // -----------------------------------------------------------------------

    fn tokenize_all(input: &[u8]) -> Vec<Token> {
        let mut t = Tokenizer::new(input);
        let mut tokens = Vec::new();
        while let Some(tok) = t.next_token() {
            tokens.push(tok);
        }
        tokens
    }

    // -----------------------------------------------------------------------
    // Number parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_integer_positive() {
        let tokens = tokenize_all(b"42");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n - 42.0).abs() < 1e-9);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_tokenizer_integer_negative() {
        let tokens = tokenize_all(b"-17");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n - (-17.0)).abs() < 1e-9);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_tokenizer_float_positive() {
        let tokens = tokenize_all(b"3.14");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n - 3.14).abs() < 1e-6);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_tokenizer_float_negative() {
        let tokens = tokenize_all(b"-0.5");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n - (-0.5)).abs() < 1e-9);
        } else {
            panic!("Expected Number token");
        }
    }

    #[test]
    fn test_tokenizer_multiple_numbers() {
        let tokens = tokenize_all(b"1 2 3");
        assert_eq!(tokens.len(), 3);
        for (i, tok) in tokens.iter().enumerate() {
            if let Token::Number(n) = tok {
                assert!((*n - (i as f64 + 1.0)).abs() < 1e-9);
            } else {
                panic!("Expected Number at index {}", i);
            }
        }
    }

    #[test]
    fn test_tokenizer_zero() {
        let tokens = tokenize_all(b"0");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n).abs() < 1e-9);
        } else {
            panic!("Expected Number");
        }
    }

    // -----------------------------------------------------------------------
    // Name parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_name_simple() {
        let tokens = tokenize_all(b"/Font");
        assert_eq!(tokens.len(), 1);
        if let Token::Name(s) = &tokens[0] {
            assert_eq!(s, "Font");
        } else {
            panic!("Expected Name token");
        }
    }

    #[test]
    fn test_tokenizer_name_with_digits() {
        let tokens = tokenize_all(b"/F1");
        assert_eq!(tokens.len(), 1);
        if let Token::Name(s) = &tokens[0] {
            assert_eq!(s, "F1");
        } else {
            panic!("Expected Name token");
        }
    }

    #[test]
    fn test_tokenizer_name_subtype() {
        let tokens = tokenize_all(b"/DeviceRGB");
        assert_eq!(tokens.len(), 1);
        if let Token::Name(s) = &tokens[0] {
            assert_eq!(s, "DeviceRGB");
        } else {
            panic!("Expected Name");
        }
    }

    // -----------------------------------------------------------------------
    // String parsing (literal)
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_literal_string_simple() {
        let tokens = tokenize_all(b"(Hello)");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"Hello");
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_literal_string_empty() {
        let tokens = tokenize_all(b"()");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert!(b.is_empty());
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_literal_string_escape_newline() {
        let tokens = tokenize_all(b"(A\\nB)");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"A\nB");
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_literal_string_escape_parens() {
        let tokens = tokenize_all(b"(a\\(b\\)c)");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"a(b)c");
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_literal_string_nested_parens() {
        let tokens = tokenize_all(b"(a(b)c)");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"a(b)c");
        } else {
            panic!("Expected StringBytes");
        }
    }

    // -----------------------------------------------------------------------
    // String parsing (hex)
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_hex_string_simple() {
        let tokens = tokenize_all(b"<48656C6C6F>"); // "Hello" in hex
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"Hello");
        } else {
            panic!("Expected StringBytes from hex");
        }
    }

    #[test]
    fn test_tokenizer_hex_string_empty() {
        let tokens = tokenize_all(b"<>");
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert!(b.is_empty());
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_hex_string_lowercase() {
        let tokens = tokenize_all(b"<ff00>"); // bytes [255, 0]
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, &[0xFF, 0x00]);
        } else {
            panic!("Expected StringBytes");
        }
    }

    #[test]
    fn test_tokenizer_hex_string_with_whitespace() {
        let tokens = tokenize_all(b"<41 42>"); // "AB"
        assert_eq!(tokens.len(), 1);
        if let Token::StringBytes(b) = &tokens[0] {
            assert_eq!(b, b"AB");
        } else {
            panic!("Expected StringBytes");
        }
    }

    // -----------------------------------------------------------------------
    // Array parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_array_open_close() {
        let tokens = tokenize_all(b"[]");
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::ArrayOpen));
        assert!(matches!(tokens[1], Token::ArrayClose));
    }

    #[test]
    fn test_tokenizer_array_with_numbers() {
        let tokens = tokenize_all(b"[1 2 3]");
        assert_eq!(tokens.len(), 5); // ArrayOpen, 1, 2, 3, ArrayClose
        assert!(matches!(tokens[0], Token::ArrayOpen));
        assert!(matches!(tokens[1], Token::Number(_)));
        assert!(matches!(tokens[4], Token::ArrayClose));
    }

    // -----------------------------------------------------------------------
    // Comment handling
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_comment_skipped() {
        let tokens = tokenize_all(b"% This is a comment\n42");
        assert_eq!(tokens.len(), 1);
        if let Token::Number(n) = &tokens[0] {
            assert!((*n - 42.0).abs() < 1e-9);
        } else {
            panic!("Expected Number after comment");
        }
    }

    #[test]
    fn test_tokenizer_comment_at_end_of_line() {
        let tokens = tokenize_all(b"10 % comment\n20");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenizer_comment_in_middle_of_stream() {
        let tokens = tokenize_all(b"BT % begin text\nET");
        // BT, ET (comment is skipped)
        assert_eq!(tokens.len(), 2);
        if let Token::Operator(op) = &tokens[0] {
            assert_eq!(op, "BT");
        } else {
            panic!("Expected Operator BT");
        }
    }

    // -----------------------------------------------------------------------
    // Operator parsing
    // -----------------------------------------------------------------------

    #[test]
    fn test_tokenizer_operator_bt_et() {
        let tokens = tokenize_all(b"BT ET");
        assert_eq!(tokens.len(), 2);
        if let Token::Operator(op) = &tokens[0] {
            assert_eq!(op, "BT");
        } else {
            panic!("Expected BT operator");
        }
        if let Token::Operator(op) = &tokens[1] {
            assert_eq!(op, "ET");
        } else {
            panic!("Expected ET operator");
        }
    }

    #[test]
    fn test_tokenizer_operator_q_and_q() {
        let tokens = tokenize_all(b"q Q");
        assert_eq!(tokens.len(), 2);
    }

    #[test]
    fn test_tokenizer_operator_re() {
        let tokens = tokenize_all(b"72 72 72 72 re");
        assert_eq!(tokens.len(), 5);
        if let Token::Operator(op) = &tokens[4] {
            assert_eq!(op, "re");
        } else {
            panic!("Expected 're' operator");
        }
    }

    #[test]
    fn test_tokenizer_operator_w() {
        let tokens = tokenize_all(b"2 w");
        assert_eq!(tokens.len(), 2);
        if let Token::Operator(op) = &tokens[1] {
            assert_eq!(op, "w");
        } else {
            panic!("Expected 'w' operator");
        }
    }

    #[test]
    fn test_tokenizer_operator_cm() {
        let tokens = tokenize_all(b"1 0 0 1 100 200 cm");
        assert_eq!(tokens.len(), 7);
        if let Token::Operator(op) = &tokens[6] {
            assert_eq!(op, "cm");
        } else {
            panic!("Expected 'cm' operator");
        }
    }

    // -----------------------------------------------------------------------
    // ContentInterpreter integration tests
    // (tests via the public interpret() API through a minimal PDF)
    // -----------------------------------------------------------------------

    fn build_minimal_pdf_with_content(content: &[u8]) -> Vec<u8> {
        let content_len = content.len();
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");
        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        let o2 = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        let o3 = out.len();
        out.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n",
        );
        let o4 = out.len();
        out.extend_from_slice(
            format!("4 0 obj\n<< /Length {} >>\nstream\n", content_len).as_bytes(),
        );
        out.extend_from_slice(content);
        out.extend_from_slice(b"endstream\nendobj\n");
        let xref = out.len();
        out.extend_from_slice(b"xref\n0 5\n");
        out.extend_from_slice(b"0000000000 65535 f \n");
        out.extend_from_slice(format!("{:010} 00000 n \n", o1).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o2).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o3).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o4).as_bytes());
        out.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n");
        out.extend_from_slice(format!("{}\n", xref).as_bytes());
        out.extend_from_slice(b"%%EOF\n");
        out
    }

    fn interpret_content(content: &[u8]) -> Vec<DrawCommand> {
        use crate::parser::PdfDocument;
        let pdf = build_minimal_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut interp = ContentInterpreter::new(&doc, &page.resources, 792.0);
        interp.interpret(&page.content).expect("interpret")
    }

    #[test]
    fn test_interpret_empty_stream_yields_no_commands() {
        let cmds = interpret_content(b"");
        assert!(
            cmds.is_empty(),
            "Empty stream should produce no draw commands"
        );
    }

    #[test]
    fn test_interpret_comment_only_yields_no_commands() {
        let cmds = interpret_content(b"% just a comment\n");
        assert!(cmds.is_empty());
    }

    #[test]
    fn test_interpret_fill_rect_yields_fill_path_command() {
        // black rect: 0 0 0 rg  72 72 144 144 re f
        let content = b"0 0 0 rg 72 72 144 144 re f\n";
        let cmds = interpret_content(content);
        assert!(
            cmds.iter()
                .any(|c| matches!(c, DrawCommand::FillPath { .. })),
            "Expected FillPath from 're f' operators"
        );
    }

    #[test]
    fn test_interpret_stroke_path_command() {
        // stroke a rectangle
        let content = b"0 0 0 RG 1 w 72 72 144 144 re S\n";
        let cmds = interpret_content(content);
        assert!(
            cmds.iter()
                .any(|c| matches!(c, DrawCommand::StrokePath { .. })),
            "Expected StrokePath from 'S' operator"
        );
    }

    #[test]
    fn test_interpret_fill_and_stroke_b_operator() {
        let content = b"0 0 0 rg 0 0 0 RG 1 w 10 10 50 50 re B\n";
        let cmds = interpret_content(content);
        // B produces both FillPath and StrokePath
        let has_fill = cmds
            .iter()
            .any(|c| matches!(c, DrawCommand::FillPath { .. }));
        let has_stroke = cmds
            .iter()
            .any(|c| matches!(c, DrawCommand::StrokePath { .. }));
        assert!(has_fill, "Expected FillPath from 'B' operator");
        assert!(has_stroke, "Expected StrokePath from 'B' operator");
    }

    #[test]
    fn test_interpret_n_clears_path_no_paint() {
        // n: end path without painting
        let content = b"10 10 100 100 re n\n";
        let cmds = interpret_content(content);
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, DrawCommand::FillPath { .. })),
            "'n' should not produce FillPath"
        );
        assert!(
            !cmds
                .iter()
                .any(|c| matches!(c, DrawCommand::StrokePath { .. })),
            "'n' should not produce StrokePath"
        );
    }

    #[test]
    fn test_interpret_q_and_q_push_pop_graphics_state() {
        // Verify that q/Q operators don't cause panics or errors
        let content = b"q 0.5 0.5 0.5 rg 10 10 50 50 re f Q 0 0 0 rg\n";
        let cmds = interpret_content(content);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, DrawCommand::FillPath { .. })));
    }

    #[test]
    fn test_interpret_cm_transform() {
        // cm sets transformation matrix
        let content = b"1 0 0 1 50 100 cm 0 0 50 50 re f\n";
        let cmds = interpret_content(content);
        assert!(cmds
            .iter()
            .any(|c| matches!(c, DrawCommand::FillPath { .. })));
    }

    #[test]
    fn test_interpret_rg_sets_fill_color() {
        // 1 0 0 rg = red fill
        let content = b"1 0 0 rg 10 10 50 50 re f\n";
        let cmds = interpret_content(content);
        let fill_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::FillPath { color, .. } = c {
                    Some(color)
                } else {
                    None
                }
            })
            .collect();
        assert!(!fill_cmds.is_empty());
        // Color should be red: r≈1.0, g≈0.0, b≈0.0
        let col = fill_cmds[0];
        assert!(col.r > 0.9, "Red channel should be near 1.0, got {}", col.r);
        assert!(
            col.g < 0.1,
            "Green channel should be near 0.0, got {}",
            col.g
        );
    }

    #[test]
    fn test_interpret_gray_fill_g_operator() {
        // 0.5 g = 50% gray fill
        let content = b"0.5 g 20 20 60 60 re f\n";
        let cmds = interpret_content(content);
        let fill_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::FillPath { color, .. } = c {
                    Some(color)
                } else {
                    None
                }
            })
            .collect();
        assert!(!fill_cmds.is_empty());
        let col = fill_cmds[0];
        // Gray: r == g == b
        assert!((col.r - col.g).abs() < 1e-3);
        assert!((col.g - col.b).abs() < 1e-3);
    }

    #[test]
    fn test_interpret_evenodd_fill_rule_f_star() {
        let content = b"0 0 0 rg 10 10 100 100 re f*\n";
        let cmds = interpret_content(content);
        let fill_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::FillPath { fill_rule, .. } = c {
                    Some(fill_rule)
                } else {
                    None
                }
            })
            .collect();
        assert!(!fill_cmds.is_empty());
        assert!(
            matches!(fill_cmds[0], FillRule::EvenOdd),
            "f* should use EvenOdd fill rule"
        );
    }

    #[test]
    fn test_interpret_nonzero_fill_rule_f_operator() {
        let content = b"0 0 0 rg 10 10 100 100 re f\n";
        let cmds = interpret_content(content);
        let fill_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::FillPath { fill_rule, .. } = c {
                    Some(fill_rule)
                } else {
                    None
                }
            })
            .collect();
        assert!(!fill_cmds.is_empty());
        assert!(
            matches!(fill_cmds[0], FillRule::NonZero),
            "f should use NonZero fill rule"
        );
    }

    #[test]
    fn test_interpret_bt_et_noop_without_font() {
        // BT/ET without font set: no glyph commands
        let content = b"BT ET\n";
        let cmds = interpret_content(content);
        assert!(
            !cmds.iter().any(|c| matches!(c, DrawCommand::DrawGlyph(_))),
            "No glyphs without font"
        );
    }

    #[test]
    fn test_interpret_line_width_w_operator() {
        // w sets line width; stroke should have the specified width
        let content = b"3 w 0 0 0 RG 10 10 100 100 re S\n";
        let cmds = interpret_content(content);
        let stroke_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::StrokePath { width, .. } = c {
                    Some(*width)
                } else {
                    None
                }
            })
            .collect();
        assert!(!stroke_cmds.is_empty());
        assert!(
            (stroke_cmds[0] - 3.0).abs() < 0.1,
            "Line width should be 3.0, got {}",
            stroke_cmds[0]
        );
    }

    #[test]
    fn test_interpret_multiple_rects_multiple_fill_commands() {
        let content = b"0 0 0 rg 10 10 50 50 re f 20 20 60 60 re f\n";
        let cmds = interpret_content(content);
        let fill_count = cmds
            .iter()
            .filter(|c| matches!(c, DrawCommand::FillPath { .. }))
            .count();
        assert_eq!(
            fill_count, 2,
            "Two 're f' should produce two FillPath commands"
        );
    }

    #[test]
    fn test_interpret_unknown_operator_does_not_panic() {
        // Unknown operators should be silently ignored (logged as trace)
        let content = b"SomeUnknownOp 42 AnotherUnknown\n";
        let result = {
            use crate::parser::PdfDocument;
            let pdf = build_minimal_pdf_with_content(content);
            let doc = PdfDocument::from_bytes(&pdf).expect("parse");
            let page = doc.get_page(0).expect("page 0");
            let mut interp = ContentInterpreter::new(&doc, &page.resources, 792.0);
            interp.interpret(&page.content)
        };
        assert!(result.is_ok(), "Unknown operators should not cause error");
    }

    // -----------------------------------------------------------------------
    // pop_* helper tests via effect in operator execution
    // -----------------------------------------------------------------------

    #[test]
    fn test_cmyk_to_rgb_conversion_k_operator() {
        // Pure black in CMYK: 0 0 0 1 k → RGB (0, 0, 0)
        let content = b"0 0 0 1 k 10 10 50 50 re f\n";
        let cmds = interpret_content(content);
        let fill_cmds: Vec<_> = cmds
            .iter()
            .filter_map(|c| {
                if let DrawCommand::FillPath { color, .. } = c {
                    Some(color)
                } else {
                    None
                }
            })
            .collect();
        assert!(!fill_cmds.is_empty());
        let col = fill_cmds[0];
        assert!(
            col.r < 0.1,
            "Pure CMYK black: r should be ~0, got {}",
            col.r
        );
    }

    #[test]
    fn test_close_and_stroke_s_operator() {
        // 's' closes path and strokes
        let content = b"0 0 0 RG 1 w 10 10 m 60 10 l 60 60 l s\n";
        let cmds = interpret_content(content);
        assert!(
            cmds.iter()
                .any(|c| matches!(c, DrawCommand::StrokePath { .. })),
            "Expected StrokePath from 's' operator"
        );
    }
}
