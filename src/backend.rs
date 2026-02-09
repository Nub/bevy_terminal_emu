use std::convert::Infallible;

use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};

/// In-memory terminal backend for Bevy integration.
///
/// This implements ratatui's `Backend` trait, storing cell data in a flat buffer.
/// A generation counter tracks when the buffer has been flushed, so the sync
/// system can skip no-op frames.
pub struct BevyBackend {
    width: u16,
    height: u16,
    pub(crate) buffer: Vec<Cell>,
    cursor: Position,
    cursor_visible: bool,
    flush_generation: u64,
}

impl BevyBackend {
    /// Create a new backend with the given dimensions.
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            buffer: vec![Cell::default(); size],
            cursor: Position { x: 0, y: 0 },
            cursor_visible: false,
            flush_generation: 0,
        }
    }

    /// Get the current flush generation counter.
    pub fn generation(&self) -> u64 {
        self.flush_generation
    }

    /// Get a reference to the internal buffer.
    pub fn buffer(&self) -> &[Cell] {
        &self.buffer
    }

    /// Get the cell at (col, row), if in bounds.
    pub fn cell(&self, col: u16, row: u16) -> Option<&Cell> {
        if col < self.width && row < self.height {
            Some(&self.buffer[row as usize * self.width as usize + col as usize])
        } else {
            None
        }
    }
}

impl Backend for BevyBackend {
    type Error = Infallible;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            if x < self.width && y < self.height {
                let idx = y as usize * self.width as usize + x as usize;
                self.buffer[idx] = cell.clone();
            }
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        self.cursor_visible = true;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        Ok(self.cursor)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> Result<(), Self::Error> {
        self.cursor = position.into();
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        for cell in &mut self.buffer {
            cell.reset();
        }
        Ok(())
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        match clear_type {
            ClearType::All => self.clear(),
            ClearType::AfterCursor => {
                let start = self.cursor.y as usize * self.width as usize + self.cursor.x as usize;
                for cell in self.buffer[start..].iter_mut() {
                    cell.reset();
                }
                Ok(())
            }
            ClearType::BeforeCursor => {
                let end = self.cursor.y as usize * self.width as usize + self.cursor.x as usize;
                let end = end.min(self.buffer.len());
                for cell in self.buffer[..end].iter_mut() {
                    cell.reset();
                }
                Ok(())
            }
            ClearType::CurrentLine => {
                let start = self.cursor.y as usize * self.width as usize;
                let end = start + self.width as usize;
                let end = end.min(self.buffer.len());
                for cell in self.buffer[start..end].iter_mut() {
                    cell.reset();
                }
                Ok(())
            }
            ClearType::UntilNewLine => {
                let start = self.cursor.y as usize * self.width as usize + self.cursor.x as usize;
                let end = (self.cursor.y as usize + 1) * self.width as usize;
                let end = end.min(self.buffer.len());
                if start < self.buffer.len() {
                    for cell in self.buffer[start..end].iter_mut() {
                        cell.reset();
                    }
                }
                Ok(())
            }
        }
    }

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(Size {
            width: self.width,
            height: self.height,
        })
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        Ok(WindowSize {
            columns_rows: Size {
                width: self.width,
                height: self.height,
            },
            pixels: Size {
                width: 0,
                height: 0,
            },
        })
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.flush_generation += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;

    #[test]
    fn test_backend_size() {
        let backend = BevyBackend::new(80, 24);
        assert_eq!(backend.size().unwrap(), Size { width: 80, height: 24 });
    }

    #[test]
    fn test_backend_draw_and_flush() {
        let mut backend = BevyBackend::new(10, 10);
        assert_eq!(backend.generation(), 0);

        let cell = Cell::default();
        backend.draw(vec![(0, 0, &cell)].into_iter()).unwrap();
        backend.flush().unwrap();
        assert_eq!(backend.generation(), 1);
    }

    #[test]
    fn test_backend_clear() {
        let mut backend = BevyBackend::new(10, 10);
        backend.clear().unwrap();
        // All cells should be default after clear
        for cell in backend.buffer() {
            assert_eq!(cell.symbol(), " ");
        }
    }

    #[test]
    fn test_backend_with_terminal() {
        let backend = BevyBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| {
                let area = frame.area();
                assert_eq!(area.width, 80);
                assert_eq!(area.height, 24);
            })
            .unwrap();
    }

    #[test]
    fn test_cursor_operations() {
        let mut backend = BevyBackend::new(80, 24);
        backend.set_cursor_position(Position { x: 5, y: 10 }).unwrap();
        assert_eq!(backend.get_cursor_position().unwrap(), Position { x: 5, y: 10 });

        backend.hide_cursor().unwrap();
        backend.show_cursor().unwrap();
    }
}
