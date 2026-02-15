use gpui::*;
use smol::lock::Mutex;
use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::thread;

/// Simple terminal implementation using std::process
/// This is a simplified implementation without full PTY support
pub struct TerminalSession {
    /// Command being run (pub for access)
    pub command: String,
    /// Process handle
    #[allow(dead_code)]
    process: Option<std::process::Child>,
    /// Output buffer
    output: Arc<Mutex<Vec<String>>>,
    /// Whether the process is still running
    running: bool,
    /// Exit code when process completes
    exit_code: Option<i32>,
}

/// Terminal size in cells
#[derive(Debug, Clone, Copy)]
pub struct TerminalSize {
    pub cols: usize,
    pub rows: usize,
    pub cell_width: usize,
    pub cell_height: usize,
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            cell_width: 8,
            cell_height: 16,
        }
    }
}

/// A single cell in the terminal
#[derive(Debug, Clone)]
pub struct CellInfo {
    pub c: char,
    pub fg: (u8, u8, u8),
    pub bg: (u8, u8, u8),
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub inverse: bool,
}

/// A line of terminal cells
#[derive(Debug, Clone)]
pub struct TerminalLine {
    pub cells: Vec<CellInfo>,
}

/// Cursor position information
#[derive(Debug, Clone)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

/// Complete terminal content for rendering
#[derive(Debug, Clone)]
pub struct TerminalContent {
    pub lines: Vec<TerminalLine>,
    pub cursor_pos: CursorPos,
    pub num_cols: usize,
    pub num_rows: usize,
}

impl TerminalSession {
    /// Create a new terminal session with the given command
    pub fn new(command: &str, _size: TerminalSize, cx: &mut Context<Self>) -> anyhow::Result<Self> {
        let output = Arc::new(Mutex::new(Vec::new()));
        let output_clone = output.clone();

        // Parse command and arguments
        let parts: Vec<&str> = command.split_whitespace().collect();
        if parts.is_empty() {
            return Err(anyhow::anyhow!("Empty command"));
        }

        let program = parts[0];
        let args = &parts[1..];

        // Start the process
        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Get stdout for output
        let stdout = child.stdout.take().unwrap();
        let _stderr = child.stderr.take().unwrap();

        // Spawn output reading thread
        thread::spawn(move || {
            let mut reader = std::io::BufReader::new(stdout);
            let mut buffer = [0u8; 1024];
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => {
                        let text = String::from_utf8_lossy(&buffer[..n]).to_string();
                        smol::block_on(async {
                            let mut output_guard = output_clone.lock().await;
                            for line in text.lines() {
                                output_guard.push(line.to_string());
                            }
                        });
                    }
                    Err(_) => break,
                }
            }
        });

        // Schedule a periodic update to refresh the terminal display
        cx.spawn(async move |this, cx| {
            loop {
                smol::Timer::after(std::time::Duration::from_millis(100)).await;
                this.update(cx, |this, cx| {
                    cx.notify();
                    if !this.running {}
                })
                .ok();
            }
        })
        .detach();

        Ok(Self {
            command: command.to_string(),
            process: Some(child),
            output,
            running: true,
            exit_code: None,
        })
    }

    /// Send input to the terminal
    pub fn input(&self, data: &[u8]) {
        // Input handling - not implemented in simplified version
        let _ = data;
    }

    /// Get visible lines for rendering
    pub async fn get_visible_content(&self) -> TerminalContent {
        let output_guard = self.output.lock().await;
        let lines: Vec<String> = output_guard.clone();
        drop(output_guard);

        let mut terminal_lines = Vec::new();

        for line_text in lines {
            let mut cells = Vec::new();
            for c in line_text.chars() {
                cells.push(CellInfo {
                    c,
                    fg: (200, 200, 200), // Light gray
                    bg: (30, 30, 30),    // Dark background
                    bold: false,
                    italic: false,
                    underline: false,
                    inverse: false,
                });
            }
            terminal_lines.push(TerminalLine { cells });
        }

        // Ensure we have at least some lines
        if terminal_lines.is_empty() {
            for _ in 0..24 {
                terminal_lines.push(TerminalLine { cells: vec![] });
            }
        }

        let num_rows = terminal_lines.len();
        let num_cols = terminal_lines.first().map(|l| l.cells.len()).unwrap_or(80);

        TerminalContent {
            lines: terminal_lines,
            cursor_pos: CursorPos {
                line: num_rows.saturating_sub(1),
                col: 0,
            },
            num_cols,
            num_rows,
        }
    }

    /// Check if process is still running
    pub fn is_running(&self) -> bool {
        self.running
    }

    /// Get exit code if process has exited
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Get current terminal size
    pub fn size(&self) -> TerminalSize {
        TerminalSize::default()
    }

    /// Resize the terminal (no-op in this simple implementation)
    pub fn resize(&mut self, _size: TerminalSize, cx: &mut Context<Self>) {
        cx.notify();
    }
}
