use std::iter::repeat;
use figlet_rs::FIGfont;
use once_cell::unsync::OnceCell;

use crate::game::Field;

pub(crate) trait Component {
    /// Render the component into the buffer, starting at the start of the slice and at the end of the Strings. If the
    /// slice is not long enough to render the component, cut off the bottom. Returns the slice, starting after the last
    /// line rendered to.
    fn render_at<'buf>(&self, buffer: &'buf mut[String]) -> &'buf mut[String];
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub(crate) struct BoxedComponent<'a, T: Component>(pub(crate) &'a T);

impl<'a, T: Component> Component for BoxedComponent<'a, T> {
    fn render_at<'buf>(&self, buffer: &'buf mut[String]) -> &'buf mut[String] {
        let buffer_len = buffer.len();

        if buffer_len < 1 {
            return buffer;
        }

        let inner_width = self.0.width();

        buffer[0].push('╭');
        buffer[0].extend(repeat('─').take(inner_width));
        buffer[0].push('╮');

        let inner_height = self.0.height();

        for line in buffer.iter_mut().skip(1).take(inner_height) {
            line.push('│');
        }

        self.0.render_at(&mut buffer[1..buffer_len.min(inner_height + 1)]);

        for line in buffer.iter_mut().skip(1).take(inner_height) {
            line.push('│');
        }

        if buffer_len > inner_height + 1 {
            buffer[inner_height + 1].push('╰');
            buffer[inner_height + 1].extend(repeat('─').take(inner_width));
            buffer[inner_height + 1].push('╯');
        }

        if buffer.len() <= inner_height + 2 {
            buffer
        } else {
            &mut buffer[inner_height + 2..]
        }
    }

    fn width(&self) -> usize {
        self.0.width() + 2
    }

    fn height(&self) -> usize {
        self.0.height() + 2
    }
}


impl Component for Field {
    fn render_at<'buf>(&self, buffer: &'buf mut[String]) -> &'buf mut[String] {
        for (dest, src) in buffer.iter_mut().zip(self.board.iter()) {
            dest.extend(src.iter().map(|cell| cell.to_string()))
        }
        let buffer_len = buffer.len();
        &mut buffer[buffer_len.min(self.board.len())..]
    }

    fn width(&self) -> usize {
        self.board.get(0).map(|row| row.len()).unwrap_or(0)
    }

    fn height(&self) -> usize {
        self.board.len()
    }
}


pub(crate) struct Controls;

impl Controls {
    const TEXT: &str = "test";

    const WIDTH: usize = {
        let mut longest = 0;
        let mut current = 0;
        let bytes = Self::TEXT.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\n' {
                if current > longest {
                    longest = current;
                }
                current = 0;
            } else if bytes[i] != b'\r' {
                current += 1;
            }
            i += 1;
        }
        if current > longest { current } else { longest }
    };

    const HEIGHT: usize = {
        let mut count = if Self::TEXT.is_empty() { 0 } else { 1 };
        let bytes = Self::TEXT.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\n' {
                count += 1;
            }
            i += 1;
        }
        count
    };
}

impl Component for Controls {
    fn render_at<'buf>(&self, buffer: &'buf mut[String]) -> &'buf mut[String] {
        for (dest, src) in buffer.iter_mut().zip(Self::TEXT.lines()) {
            dest.push_str(src);
        }
        let buffer_len = buffer.len();
        &mut buffer[buffer_len.min(Self::HEIGHT)..]
    }

    fn width(&self) -> usize {
        Self::WIDTH
    }

    fn height(&self) -> usize {
        Self::HEIGHT
    }
}


pub(crate) struct Title<'a> {
    text: &'a str,
    title: OnceCell<String>,
}

impl<'a> Title<'a> {
    pub(crate) fn new(text: &'a str) -> Self {
        Self { text, title: OnceCell::new() }
    }

    fn lazy_title(&self) -> &str {
        self.title.get_or_init(|| FIGfont::standard()
            .unwrap()
            .convert(self.text)
            .unwrap()
            .to_string()
        )
    }
}

impl<'a> Component for Title<'a> {
    fn render_at<'buf>(&self, buffer: &'buf mut[String]) -> &'buf mut[String] {
        let mut count = 0;
        for (dest, src) in buffer.iter_mut().zip(self.text.lines()) {
            count += 1;
            dest.push_str(src);
        }
        let buffer_len = buffer.len();
        &mut buffer[buffer_len.min(count)..]
    }

    fn width(&self) -> usize {
        self.lazy_title()
            .lines()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
    }

    fn height(&self) -> usize {
        self.lazy_title()
            .bytes()
            .filter(|c| *c == b'\n')
            .count()
    }
}
