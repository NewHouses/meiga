//! The terminal of `alacritty_terminal` as the parser sees it: it passes
//! every action of the output to it, and when the program leaves the
//! alternate screen, or resets the terminal, it gives back the primary
//! screen that a resize set aside ([`super::resize`]).
//!
//! It has to be there at the moment the program leaves the alternate
//! screen: the output that comes after it, in the same read or in the same
//! synchronized update (`?2026`), already goes to the primary screen.
//!
//! It also keeps the view of the history where it is when a scrolling
//! region that starts below the first row of the screen scrolls: its rows do
//! not go into the history, but `alacritty_terminal` moves the view up as if
//! they did.

use alacritty_terminal::grid::{Dimensions, Grid, Scroll};
use alacritty_terminal::term::cell::Cell;
use alacritty_terminal::term::{Term, TermMode};
use alacritty_terminal::vte::ansi::cursor_icon::CursorIcon;
use alacritty_terminal::vte::ansi::{
    Attr, CharsetIndex, ClearMode, CursorShape, CursorStyle, Handler, Hyperlink, KeyboardModes,
    KeyboardModesApplyBehavior, LineClearMode, Mode, ModifyOtherKeys, NamedPrivateMode,
    PrivateMode, Rgb, ScpCharPath, ScpUpdateMode, StandardCharset, TabulationClearMode,
};

use super::{Listener, resize};

/// The terminal, and the primary screen that a resize set aside while the
/// alternate one is shown.
pub(super) struct Hooked<'a> {
    pub term: &'a mut Term<Listener>,
    pub primary: &'a mut Option<Grid<Cell>>,
    /// The scrolling region of the program starts below the first row of the
    /// screen. `alacritty_terminal` does not tell where it starts: it is
    /// followed here.
    pub region_below_top: &'a mut bool,
}

impl Hooked<'_> {
    /// How far above the screen the view of the history is, if an action
    /// that scrolls from a row below the first one, `below_top`, could move
    /// it.
    #[inline]
    fn view_to_keep(&self, below_top: bool) -> Option<usize> {
        if !below_top {
            return None;
        }
        let offset = self.term.grid().display_offset();
        (offset != 0).then_some(offset)
    }

    /// Puts the view back at `view`, if the action moved it.
    #[inline]
    fn keep_view(&mut self, view: Option<usize>) {
        let Some(offset) = view else {
            return;
        };
        let moved = self.term.grid().display_offset();
        if moved != offset {
            // Both are at most SCROLLBACK_LINES: they fit in an i32.
            self.term
                .scroll_display(Scroll::Delta(offset as i32 - moved as i32));
        }
    }

    /// Follows the scrolling region that `alacritty_terminal` sets from the
    /// row `top`, where the first one is 1, to `bottom`, or to the last row:
    /// it ignores it if `top` is not above that row.
    #[inline]
    fn follow_region(&mut self, top: usize, bottom: Option<usize>) {
        if top < bottom.unwrap_or_else(|| self.term.screen_lines()) {
            *self.region_below_top = top > 1;
        }
    }

    /// DECCOLM (`?3h` and `?3l`): `alacritty_terminal` keeps the columns,
    /// but it clears the screen and sets the region to the whole screen, as
    /// `CSI 1 r`, inside the terminal, without this handler.
    #[inline]
    fn follow_column_mode(&mut self, mode: PrivateMode) {
        if matches!(mode, PrivateMode::Named(NamedPrivateMode::ColumnMode)) {
            self.follow_region(1, None);
        }
    }
}

/// The methods of [`Handler`] that only pass the action to the terminal.
macro_rules! pass {
    ($(fn $name:ident(&mut self $(, $argument:ident: $type:ty)*);)*) => {
        $(
            #[inline]
            fn $name(&mut self $(, $argument: $type)*) {
                <Term<Listener> as Handler>::$name(self.term $(, $argument)*);
            }
        )*
    };
}

// A method that Handler adds in a new version would do nothing here, without
// any error: the lint asks for all of them.
#[deny(clippy::missing_trait_methods)]
impl Handler for Hooked<'_> {
    pass! {
        fn set_title(&mut self, title: Option<String>);
        fn set_cursor_style(&mut self, style: Option<CursorStyle>);
        fn set_cursor_shape(&mut self, shape: CursorShape);
        fn goto(&mut self, line: i32, column: usize);
        fn goto_line(&mut self, line: i32);
        fn goto_col(&mut self, column: usize);
        fn insert_blank(&mut self, count: usize);
        fn move_up(&mut self, rows: usize);
        fn move_down(&mut self, rows: usize);
        fn identify_terminal(&mut self, intermediate: Option<char>);
        fn device_status(&mut self, kind: usize);
        fn move_forward(&mut self, columns: usize);
        fn move_backward(&mut self, columns: usize);
        fn move_down_and_cr(&mut self, rows: usize);
        fn move_up_and_cr(&mut self, rows: usize);
        fn backspace(&mut self);
        fn carriage_return(&mut self);
        fn bell(&mut self);
        fn substitute(&mut self);
        fn set_horizontal_tabstop(&mut self);
        fn scroll_down(&mut self, rows: usize);
        fn insert_blank_lines(&mut self, count: usize);
        fn erase_chars(&mut self, count: usize);
        fn delete_chars(&mut self, count: usize);
        fn move_backward_tabs(&mut self, count: u16);
        fn move_forward_tabs(&mut self, count: u16);
        fn save_cursor_position(&mut self);
        fn restore_cursor_position(&mut self);
        fn clear_line(&mut self, mode: LineClearMode);
        fn clear_screen(&mut self, mode: ClearMode);
        fn clear_tabs(&mut self, mode: TabulationClearMode);
        fn set_tabs(&mut self, interval: u16);
        fn reverse_index(&mut self);
        fn terminal_attribute(&mut self, attribute: Attr);
        fn set_mode(&mut self, mode: Mode);
        fn unset_mode(&mut self, mode: Mode);
        fn report_mode(&mut self, mode: Mode);
        fn report_private_mode(&mut self, mode: PrivateMode);
        fn set_keypad_application_mode(&mut self);
        fn unset_keypad_application_mode(&mut self);
        fn set_active_charset(&mut self, index: CharsetIndex);
        fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset);
        fn set_color(&mut self, index: usize, color: Rgb);
        fn dynamic_color_sequence(&mut self, prefix: String, index: usize, terminator: &str);
        fn reset_color(&mut self, index: usize);
        fn clipboard_store(&mut self, clipboard: u8, base64: &[u8]);
        fn clipboard_load(&mut self, clipboard: u8, terminator: &str);
        fn decaln(&mut self);
        fn push_title(&mut self);
        fn pop_title(&mut self);
        fn text_area_size_pixels(&mut self);
        fn text_area_size_chars(&mut self);
        fn set_hyperlink(&mut self, hyperlink: Option<Hyperlink>);
        fn set_mouse_cursor_icon(&mut self, icon: CursorIcon);
        fn report_keyboard_mode(&mut self);
        fn push_keyboard_mode(&mut self, mode: KeyboardModes);
        fn pop_keyboard_modes(&mut self, count: u16);
        fn set_keyboard_mode(&mut self, mode: KeyboardModes, behavior: KeyboardModesApplyBehavior);
        fn set_modify_other_keys(&mut self, mode: ModifyOtherKeys);
        fn report_modify_other_keys(&mut self);
        fn set_scp(&mut self, path: ScpCharPath, update: ScpUpdateMode);
    }

    /// A character. After the last column, it goes to the next row, which
    /// scrolls the region at its bottom.
    #[inline]
    fn input(&mut self, c: char) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::input(self.term, c);
        self.keep_view(view);
    }

    /// A tab after the last column goes to the next row, like a character.
    #[inline]
    fn put_tab(&mut self, count: u16) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::put_tab(self.term, count);
        self.keep_view(view);
    }

    /// LF, VT, FF, `ESC D` and `ESC E`: at the bottom of the region, it
    /// scrolls.
    #[inline]
    fn linefeed(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::linefeed(self.term);
        self.keep_view(view);
    }

    /// A line feed and, in the mode `LNM`, a carriage return. The parser
    /// does not use it.
    #[inline]
    fn newline(&mut self) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::newline(self.term);
        self.keep_view(view);
    }

    /// `CSI n S`: the region scrolls up `rows` rows.
    #[inline]
    fn scroll_up(&mut self, rows: usize) {
        let view = self.view_to_keep(*self.region_below_top);
        <Term<Listener> as Handler>::scroll_up(self.term, rows);
        self.keep_view(view);
    }

    /// `CSI n M`: the rows of the region below the cursor move up, from the
    /// row of the cursor.
    #[inline]
    fn delete_lines(&mut self, count: usize) {
        let view = self.view_to_keep(self.term.grid().cursor.point.line.0 != 0);
        <Term<Listener> as Handler>::delete_lines(self.term, count);
        self.keep_view(view);
    }

    /// `CSI top ; bottom r`: the scrolling region, from the row `top`, where
    /// the first one is 1.
    #[inline]
    fn set_scrolling_region(&mut self, top: usize, bottom: Option<usize>) {
        <Term<Listener> as Handler>::set_scrolling_region(self.term, top, bottom);
        self.follow_region(top, bottom);
    }

    /// `?3h` is DECCOLM, which also sets the scrolling region.
    #[inline]
    fn set_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::set_private_mode(self.term, mode);
        self.follow_column_mode(mode);
    }

    /// `?3l` is DECCOLM, as `?3h`. `?1049l`: back to the primary screen,
    /// with the size of the terminal. A new size also resets the scrolling
    /// region.
    #[inline]
    fn unset_private_mode(&mut self, mode: PrivateMode) {
        <Term<Listener> as Handler>::unset_private_mode(self.term, mode);
        self.follow_column_mode(mode);
        if !self.term.mode().contains(TermMode::ALT_SCREEN)
            && let Some(primary) = self.primary.take()
            && resize::bring_back_primary(self.term, primary)
        {
            *self.region_below_top = false;
        }
    }

    /// `ESC c`: the reset also leaves the alternate screen, and empties the
    /// primary one, history included. The scrolling region is the whole
    /// screen again.
    #[inline]
    fn reset_state(&mut self) {
        <Term<Listener> as Handler>::reset_state(self.term);
        *self.region_below_top = false;
        if self.primary.take().is_some() {
            resize::new_primary(self.term);
        }
    }
}
