#![allow(static_mut_refs)]

use std::time::Instant;
use std::ptr::{self, null_mut};

const COLS: usize = 10;
const ROWS: usize = 20;
const CELL: i32 = 28;
const SIDE: i32 = 120;
const BOARD_X: i32 = 8;
const BOARD_Y: i32 = 8;
const WIN_W: i32 = BOARD_X * 2 + COLS as i32 * CELL + SIDE;
const WIN_H: i32 = BOARD_Y * 2 + ROWS as i32 * CELL;
const WIN_WU: usize = WIN_W as usize;
const WIN_HU: usize = WIN_H as usize;

const COLORS: [u32; 8] = [
    0x00000000,
    0x00DDDD00,
    0x0000DDDD,
    0x00AA00AA,
    0x0000DD00,
    0x00DD0000,
    0x000000DD,
    0x00DD7700,
];

const SHAPES: [[[i8; 8]; 4]; 7] = [
    [[0,1,1,1,2,1,3,1],[2,0,2,1,2,2,2,3],[0,2,1,2,2,2,3,2],[1,0,1,1,1,2,1,3]],
    [[0,0,1,0,0,1,1,1],[0,0,1,0,0,1,1,1],[0,0,1,0,0,1,1,1],[0,0,1,0,0,1,1,1]],
    [[0,1,1,1,2,1,1,0],[1,0,1,1,1,2,2,1],[0,1,1,1,2,1,1,2],[1,0,1,1,1,2,0,1]],
    [[1,0,2,0,0,1,1,1],[1,0,1,1,2,1,2,2],[1,1,2,1,0,2,1,2],[0,0,0,1,1,1,1,2]],
    [[0,0,1,0,1,1,2,1],[2,0,1,1,2,1,1,2],[0,1,1,1,1,2,2,2],[1,0,0,1,1,1,0,2]],
    [[0,0,0,1,1,1,2,1],[1,0,2,0,1,1,1,2],[0,1,1,1,2,1,2,2],[1,0,1,1,0,2,1,2]],
    [[2,0,0,1,1,1,2,1],[1,0,1,1,1,2,2,2],[0,1,1,1,2,1,0,2],[0,0,1,0,1,1,1,2]],
];

struct Game {
    board: [u8; COLS * ROWS],
    px: i32,
    py: i32,
    piece: u8,
    rot: u8,
    next: u8,
    score: u32,
    lines: u32,
    level: u32,
    over: bool,
    rng: u32,
    last_drop: Instant,
    drop_ms: u64,
    need_redraw: bool,
}

impl Game {
    fn new() -> Self {
        let mut g = Game {
            board: [0; COLS * ROWS],
            px: 0, py: 0,
            piece: 0, rot: 0,
            next: 0,
            score: 0, lines: 0, level: 1,
            over: false,
            rng: 0xDEAD_BEEF,
            last_drop: Instant::now(),
            drop_ms: 800,
            need_redraw: true,
        };
        g.next = (g.rand() % 7) as u8;
        g.spawn();
        g
    }

    fn rand(&mut self) -> u32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 17;
        self.rng ^= self.rng << 5;
        self.rng
    }

    fn spawn(&mut self) {
        self.piece = self.next;
        self.rot = 0;
        self.px = 3;
        self.py = 0;
        self.next = (self.rand() % 7) as u8;
        if !self.valid(self.piece, self.rot, self.px, self.py) {
            self.over = true;
        }
        self.need_redraw = true;
    }

    fn block_at(piece: u8, rot: u8, idx: usize) -> (i32, i32) {
        let d = &SHAPES[piece as usize][rot as usize];
        (d[idx * 2] as i32, d[idx * 2 + 1] as i32)
    }

    fn valid(&self, piece: u8, rot: u8, px: i32, py: i32) -> bool {
        for i in 0..4 {
            let (bx, by) = Self::block_at(piece, rot, i);
            let x = px + bx;
            let y = py + by;
            if x < 0 || x >= COLS as i32 || y < 0 || y >= ROWS as i32 {
                return false;
            }
            if self.board[y as usize * COLS + x as usize] != 0 {
                return false;
            }
        }
        true
    }

    fn lock(&mut self) {
        for i in 0..4 {
            let (bx, by) = Self::block_at(self.piece, self.rot, i);
            let x = self.px + bx;
            let y = self.py + by;
            if x >= 0 && x < COLS as i32 && y >= 0 && y < ROWS as i32 {
                self.board[y as usize * COLS + x as usize] = self.piece + 1;
            }
        }
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut dst = ROWS as i32 - 1;
        for src in (0..ROWS as i32).rev() {
            let mut full = true;
            for x in 0..COLS {
                if self.board[src as usize * COLS + x] == 0 {
                    full = false;
                    break;
                }
            }
            if full {
                cleared += 1;
            } else {
                if dst != src {
                    for x in 0..COLS {
                        self.board[dst as usize * COLS + x] = self.board[src as usize * COLS + x];
                    }
                }
                dst -= 1;
            }
        }
        for i in 0..=(dst as usize * COLS + COLS - 1).min(COLS * ROWS - 1) {
            self.board[i] = 0;
        }
        if cleared > 0 {
            self.score += match cleared {
                1 => 100 * self.level,
                2 => 300 * self.level,
                3 => 500 * self.level,
                4 => 800 * self.level,
                _ => 0,
            };
            self.lines += cleared;
            self.level = self.lines / 10 + 1;
            self.drop_ms = (800u64).saturating_sub(((self.level - 1) as u64) * 70).max(50);
            self.need_redraw = true;
        }
    }

    fn try_move(&mut self, dx: i32, dy: i32) -> bool {
        if self.valid(self.piece, self.rot, self.px + dx, self.py + dy) {
            self.px += dx;
            self.py += dy;
            self.need_redraw = true;
            true
        } else {
            false
        }
    }

    fn rotate(&mut self) {
        let nr = (self.rot + 1) % 4;
        if self.valid(self.piece, nr, self.px, self.py) {
            self.rot = nr;
            self.need_redraw = true;
            return;
        }
        for &dx in &[-1i32, 1, -2, 2] {
            if self.valid(self.piece, nr, self.px + dx, self.py) {
                self.px += dx;
                self.rot = nr;
                self.need_redraw = true;
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        while self.try_move(0, 1) {}
        self.score += 2;
        self.lock();
        self.clear_lines();
        self.spawn();
    }

    fn tick(&mut self) {
        if self.over { return; }
        if !self.try_move(0, 1) {
            self.lock();
            self.clear_lines();
            self.spawn();
        }
    }

    fn restart(&mut self) {
        self.board = [0; COLS * ROWS];
        self.score = 0;
        self.lines = 0;
        self.level = 1;
        self.over = false;
        self.drop_ms = 800;
        self.last_drop = Instant::now();
        self.spawn();
        self.need_redraw = true;
    }
}

static mut GAME: Option<Game> = None;
static mut H_WND: *mut u8 = null_mut();
static mut H_DC: *mut u8 = null_mut();
static mut BUF: Vec<u32> = Vec::new();

unsafe fn g() -> &'static mut Game {
    unsafe { GAME.as_mut().unwrap_unchecked() }
}

unsafe fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

extern "system" {
    fn RegisterClassExW(lpwcx: *const u8) -> u16;
    fn CreateWindowExW(
        dwExStyle: u32, lpClassName: *const u16, lpWindowName: *const u16,
        dwStyle: u32, x: i32, y: i32, nWidth: i32, nHeight: i32,
        hWndParent: *mut u8, hMenu: *mut u8, hInstance: *mut u8, lpParam: *mut u8,
    ) -> *mut u8;
    fn DefWindowProcW(hWnd: *mut u8, msg: u32, wParam: usize, lParam: isize) -> isize;
    fn TranslateMessage(lpMsg: *const u8) -> i32;
    fn DispatchMessageW(lpMsg: *const u8) -> isize;
    fn GetDC(hWnd: *mut u8) -> *mut u8;
    fn ReleaseDC(hWnd: *mut u8, hDC: *mut u8) -> i32;
    fn StretchDIBits(
        hdc: *mut u8, xDest: i32, yDest: i32, w: i32, h: i32,
        xSrc: i32, ySrc: i32, wSrc: i32, hSrc: i32,
        lpBits: *const u8, lpbmi: *const u8, usage: u32, rop: u32,
    ) -> i32;
    fn PostQuitMessage(nExitCode: i32);
    fn GetModuleHandleW(lpModuleName: *const u16) -> *mut u8;
    fn PeekMessageW(lpMsg: *mut u8, hWnd: *mut u8, wMsgFilterMin: u32, wMsgFilterMax: u32, wRemoveMsg: u32) -> i32;
    fn Sleep(dwMilliseconds: u32);
}

unsafe extern "system" fn wnd_proc(
    hwnd: *mut u8, msg: u32, wparam: usize, _lparam: isize,
) -> isize {
    match msg {
        0x0002 | 0x0010 => {
            PostQuitMessage(0);
            0
        }
        0x0100 => {
            let game = g();
            if wparam == 0x52 && game.over {
                game.restart();
            } else if !game.over {
                match wparam {
                    0x25 => { game.try_move(-1, 0); }
                    0x27 => { game.try_move(1, 0); }
                    0x28 => {
                        if game.try_move(0, 1) {
                            game.score += 1;
                        }
                    }
                    0x26 => { game.rotate(); }
                    0x20 => { game.hard_drop(); }
                    _ => {}
                }
            }
            if game.need_redraw {
                render_to_buf();
                present_buffer();
                game.need_redraw = false;
            }
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, _lparam),
    }
}

unsafe fn render_to_buf() {
    let game = g();
    let bw = WIN_WU;
    let buf = BUF.as_mut_slice();

    for p in buf.iter_mut() { *p = 0xFF16213E; }

    for c in 0..COLS as i32 {
        for r in 0..ROWS as i32 {
            let color = game.board[r as usize * COLS + c as usize];
            let x = BOARD_X + c * CELL;
            let y = BOARD_Y + r * CELL;
            let col = if color > 0 { COLORS[color as usize] } else { 0xFF0F3460 };
            fill_rect(buf, bw as i32, x, y, CELL - 1, CELL - 1, col | 0xFF000000);
        }
    }

    if !game.over {
        for i in 0..4 {
            let (bx, by) = Game::block_at(game.piece, game.rot, i);
            let x = BOARD_X + (game.px + bx) * CELL;
            let y = BOARD_Y + (game.py + by) * CELL;
            let col = COLORS[game.piece as usize + 1];
            fill_rect(buf, bw as i32, x, y, CELL - 1, CELL - 1, col | 0xFF000000);
            let r = ((col >> 16) & 0xFF) as u32;
            let g_c = ((col >> 8) & 0xFF) as u32;
            let b = (col & 0xFF) as u32;
            let hi = 0xFF000000
                | ((r + 80).min(255) << 16)
                | ((g_c + 80).min(255) << 8)
                | ((b + 80).min(255));
            fill_rect(buf, bw as i32, x + 2, y + 2, CELL - 5, CELL - 5, hi);
        }
    }

    let sx = BOARD_X + COLS as i32 * CELL + 16;
    let mut sy = BOARD_Y;

    draw_str(buf, bw as i32, sx, sy, "NEXT", 0xFFCCCCCC);
    sy += 20;

    for i in 0..4 {
        let (bx, by) = Game::block_at(game.next, 0, i);
        let col = COLORS[game.next as usize + 1];
        fill_rect(buf, bw as i32, sx + bx * 20, sy + by * 20, 18, 18, col | 0xFF000000);
    }
    sy += 4 * 20 + 10;

    draw_str(buf, bw as i32, sx, sy, "SCORE", 0xFFCCCCCC);
    sy += 16;
    draw_num(buf, bw as i32, sx, sy, game.score);
    sy += 24;

    draw_str(buf, bw as i32, sx, sy, "LINES", 0xFFCCCCCC);
    sy += 16;
    draw_num(buf, bw as i32, sx, sy, game.lines);
    sy += 24;

    draw_str(buf, bw as i32, sx, sy, "LEVEL", 0xFFCCCCCC);
    sy += 16;
    draw_num(buf, bw as i32, sx, sy, game.level);

    if game.over {
        fill_rect(buf, bw as i32, 0, WIN_H / 2 - 30, WIN_W, 60, 0xCC000000);
        draw_str(buf, bw as i32, BOARD_X + 20, BOARD_Y + ROWS as i32 * CELL / 2 - 20, "GAME OVER", 0xFFFF4444);
        draw_str(buf, bw as i32, BOARD_X + 10, BOARD_Y + ROWS as i32 * CELL / 2 + 5, "PRESS R", 0xFFCCCCCC);
    }
}

unsafe fn present_buffer() {
    let mut bmi = [0u32; 11];
    bmi[0] = 40;
    bmi[1] = WIN_WU as u32;
    bmi[2] = (-WIN_H) as u32;
    bmi[3] = 1 | (32 << 16);

    StretchDIBits(
        H_DC,
        0, 0, WIN_W, WIN_H,
        0, 0, WIN_W, WIN_H,
        BUF.as_ptr() as *const u8,
        bmi.as_ptr() as *const u8,
        0, 0x00CC0020,
    );
}

fn fill_rect(buf: &mut [u32], bw: i32, x: i32, y: i32, w: i32, h: i32, color: u32) {
    for dy in 0..h {
        for dx in 0..w {
            let px = x + dx;
            let py = y + dy;
            if px >= 0 && py >= 0 && px < bw && py < WIN_H {
                buf[(py * bw + px) as usize] = color;
            }
        }
    }
}

const GLYPHS: [[u8; 5]; 95] = [
    [0x00,0x00,0x00,0x00,0x00],
    [0x00,0x00,0x5F,0x00,0x00],
    [0x00,0x07,0x00,0x07,0x00],
    [0x14,0x7F,0x14,0x7F,0x14],
    [0x24,0x2A,0x7F,0x2A,0x12],
    [0x23,0x13,0x08,0x64,0x62],
    [0x36,0x49,0x55,0x22,0x50],
    [0x00,0x05,0x03,0x00,0x00],
    [0x00,0x1C,0x22,0x41,0x00],
    [0x00,0x41,0x22,0x1C,0x00],
    [0x14,0x08,0x3E,0x08,0x14],
    [0x08,0x08,0x3E,0x08,0x08],
    [0x00,0x50,0x30,0x00,0x00],
    [0x08,0x08,0x08,0x08,0x08],
    [0x00,0x60,0x60,0x00,0x00],
    [0x20,0x10,0x08,0x04,0x02],
    [0x3E,0x51,0x49,0x45,0x3E],
    [0x00,0x42,0x7F,0x40,0x00],
    [0x42,0x61,0x51,0x49,0x46],
    [0x21,0x41,0x45,0x4B,0x31],
    [0x18,0x14,0x12,0x7F,0x10],
    [0x27,0x45,0x45,0x45,0x39],
    [0x3C,0x4A,0x49,0x49,0x30],
    [0x01,0x71,0x09,0x05,0x03],
    [0x36,0x49,0x49,0x49,0x36],
    [0x06,0x49,0x49,0x29,0x1E],
    [0x00,0x36,0x36,0x00,0x00],
    [0x00,0x56,0x36,0x00,0x00],
    [0x08,0x14,0x22,0x41,0x00],
    [0x14,0x14,0x14,0x14,0x14],
    [0x00,0x41,0x22,0x14,0x08],
    [0x02,0x01,0x51,0x09,0x06],
    [0x32,0x49,0x79,0x41,0x3E],
    [0x7E,0x11,0x11,0x11,0x7E],
    [0x7F,0x49,0x49,0x49,0x36],
    [0x3E,0x41,0x41,0x41,0x22],
    [0x7F,0x41,0x41,0x22,0x1C],
    [0x7F,0x49,0x49,0x49,0x41],
    [0x7F,0x09,0x09,0x09,0x01],
    [0x3E,0x41,0x49,0x49,0x7A],
    [0x7F,0x08,0x08,0x08,0x7F],
    [0x00,0x41,0x7F,0x41,0x00],
    [0x20,0x40,0x41,0x3F,0x01],
    [0x7F,0x08,0x14,0x22,0x41],
    [0x7F,0x40,0x40,0x40,0x40],
    [0x7F,0x02,0x0C,0x02,0x7F],
    [0x7F,0x04,0x08,0x10,0x7F],
    [0x3E,0x41,0x41,0x41,0x3E],
    [0x7F,0x09,0x09,0x09,0x06],
    [0x3E,0x41,0x51,0x21,0x5E],
    [0x7F,0x09,0x19,0x29,0x46],
    [0x46,0x49,0x49,0x49,0x31],
    [0x01,0x01,0x7F,0x01,0x01],
    [0x3F,0x40,0x40,0x40,0x3F],
    [0x1F,0x20,0x40,0x20,0x1F],
    [0x3F,0x40,0x38,0x40,0x3F],
    [0x63,0x14,0x08,0x14,0x63],
    [0x07,0x08,0x70,0x08,0x07],
    [0x61,0x51,0x49,0x45,0x43],
    [0x00,0x7F,0x41,0x41,0x00],
    [0x02,0x04,0x08,0x10,0x20],
    [0x00,0x41,0x41,0x7F,0x00],
    [0x04,0x02,0x01,0x02,0x04],
    [0x40,0x40,0x40,0x40,0x40],
    [0x00,0x01,0x02,0x04,0x00],
    [0x20,0x54,0x54,0x54,0x78],
    [0x7F,0x48,0x44,0x44,0x38],
    [0x38,0x44,0x44,0x44,0x20],
    [0x38,0x44,0x44,0x48,0x7F],
    [0x38,0x54,0x54,0x54,0x18],
    [0x08,0x7E,0x09,0x01,0x02],
    [0x0C,0x52,0x52,0x52,0x3E],
    [0x7F,0x08,0x04,0x04,0x78],
    [0x00,0x44,0x7D,0x40,0x00],
    [0x20,0x40,0x44,0x3D,0x00],
    [0x7F,0x10,0x28,0x44,0x00],
    [0x00,0x41,0x7F,0x40,0x00],
    [0x7C,0x04,0x18,0x04,0x78],
    [0x7C,0x08,0x04,0x04,0x78],
    [0x38,0x44,0x44,0x44,0x38],
    [0x7C,0x14,0x14,0x14,0x08],
    [0x08,0x14,0x14,0x18,0x7C],
    [0x7C,0x08,0x04,0x04,0x08],
    [0x48,0x54,0x54,0x54,0x20],
    [0x04,0x3F,0x44,0x40,0x20],
    [0x3C,0x40,0x40,0x20,0x7C],
    [0x1C,0x20,0x40,0x20,0x1C],
    [0x3C,0x40,0x30,0x40,0x3C],
    [0x44,0x28,0x10,0x28,0x44],
    [0x0C,0x50,0x50,0x50,0x3C],
    [0x44,0x64,0x54,0x4C,0x44],
    [0x00,0x08,0x36,0x41,0x00],
    [0x00,0x00,0x7F,0x00,0x00],
    [0x00,0x41,0x36,0x08,0x00],
    [0x10,0x08,0x08,0x10,0x08],
];

fn draw_char(buf: &mut [u32], bw: i32, x: i32, y: i32, ch: u8, color: u32, scale: i32) {
    let idx = (ch as i32) - (b' ' as i32);
    if idx < 0 || idx >= 95 { return; }
    let glyph = &GLYPHS[idx as usize];
    for row in 0..5i32 {
        let bits = glyph[row as usize];
        for col in 0..7i32 {
            if (bits >> (6 - col)) & 1 != 0 {
                fill_rect(buf, bw, x + col * scale, y + row * scale, scale, scale, color);
            }
        }
    }
}

fn draw_str(buf: &mut [u32], bw: i32, x: i32, y: i32, s: &str, color: u32) {
    let mut cx = x;
    for &ch in s.as_bytes() {
        draw_char(buf, bw, cx, y, ch, color, 2);
        cx += 12;
    }
}

fn draw_num(buf: &mut [u32], bw: i32, x: i32, y: i32, mut val: u32) {
    let mut digits = [0u8; 10];
    let mut len = 0;
    if val == 0 {
        digits[0] = b'0';
        len = 1;
    } else {
        while val > 0 && len < 10 {
            digits[len] = b'0' + (val % 10) as u8;
            val /= 10;
            len += 1;
        }
        digits[..len].reverse();
    }
    let mut cx = x;
    for i in 0..len {
        draw_char(buf, bw, cx, y, digits[i], 0xFFFFFFFF, 2);
        cx += 12;
    }
}

fn main() {
    unsafe {
        GAME = Some(Game::new());

        let hinst = GetModuleHandleW(ptr::null());

        let class_name = to_wide("TetrisCls\0");
        let title = to_wide("Tetris\0");

        #[repr(C)]
        struct WNDCLASSEXW {
            cb_size: u32,
            style: u32,
            lpfn_wnd_proc: usize,
            cb_cls_extra: i32,
            cb_wnd_extra: i32,
            h_instance: *mut u8,
            h_icon: *mut u8,
            h_cursor: *mut u8,
            hbr_background: *mut u8,
            lpsz_menu_name: *const u16,
            lpsz_class_name: *const u16,
            h_icon_sm: *mut u8,
        }

        let wc = WNDCLASSEXW {
            cb_size: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfn_wnd_proc: wnd_proc as *const () as usize,
            cb_cls_extra: 0,
            cb_wnd_extra: 0,
            h_instance: hinst,
            h_icon: null_mut(),
            h_cursor: null_mut(),
            hbr_background: null_mut(),
            lpsz_menu_name: ptr::null(),
            lpsz_class_name: class_name.as_ptr(),
            h_icon_sm: null_mut(),
        };

        RegisterClassExW(&wc as *const _ as *const u8);

        let style = 0x10CF0000u32; // WS_VISIBLE | WS_OVERLAPPEDWINDOW

        H_WND = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            style,
            0x80000000u32 as i32,
            0x80000000u32 as i32,
            WIN_W,
            WIN_H,
            null_mut(),
            null_mut(),
            hinst,
            null_mut(),
        );

        H_DC = GetDC(H_WND);

        BUF = vec![0u32; WIN_WU * WIN_HU];

        render_to_buf();
        present_buffer();

        let mut msg = [0u8; 48];
        loop {
            while PeekMessageW(msg.as_mut_ptr(), null_mut(), 0, 0, 1) != 0 {
                let msg_u32 = *(msg.as_ptr() as *const u32);
                if msg_u32 == 0x0012 {
                    ReleaseDC(H_WND, H_DC);
                    return;
                }
                TranslateMessage(msg.as_ptr());
                DispatchMessageW(msg.as_ptr());
            }

            let game = g();
            let now = Instant::now();
            if !game.over {
                let elapsed = now.duration_since(game.last_drop).as_millis() as u64;
                if elapsed >= game.drop_ms {
                    game.tick();
                    game.last_drop = now;
                    if game.need_redraw {
                        render_to_buf();
                        present_buffer();
                        game.need_redraw = false;
                    }
                }
                let remaining = game.drop_ms.saturating_sub(now.duration_since(game.last_drop).as_millis() as u64);
                Sleep((remaining.min(16) + 1) as u32);
            } else {
                Sleep(50);
            }
        }
    }
}
