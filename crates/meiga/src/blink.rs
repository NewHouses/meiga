//! When a caret that blinks is shown, as the caret of Windows: the cursor of
//! the terminal (ADR 0013), and the caret of the name of a tab while it is
//! edited (ADR 0031).

use std::time::Instant;

use meiga_win::window::CaretSettings;

/// The dots per inch at a scale of 100 %, where the width of the caret is
/// in pixels.
const BASE_DPI: u32 = 96;

/// Whether a caret that started blinking at `since`, with `settings`, is
/// shown at `now`, and when that changes next. It only blinks if `blinks`
/// and Windows allows it, and only until the timeout of the caret after
/// `since`: then it stays shown, and nothing changes any more.
pub fn phase(
    settings: &CaretSettings,
    blinks: bool,
    since: Instant,
    now: Instant,
) -> (bool, Option<Instant>) {
    let Some(period) = settings.blink.filter(|_| blinks) else {
        return (true, None);
    };
    let Some(end) = since.checked_add(settings.timeout) else {
        return (true, None);
    };
    if now >= end || period.is_zero() {
        return (true, None);
    }
    let elapsed = now.saturating_duration_since(since);
    // The halves of a blink so far: fewer than ten with the defaults of
    // Windows, and they fit in a u32 with any timeout of Windows, in
    // milliseconds.
    let halves = u32::try_from(elapsed.as_nanos() / period.as_nanos()).unwrap_or(u32::MAX);
    let next = period
        .checked_mul(halves.saturating_add(1))
        .and_then(|offset| since.checked_add(offset))
        .map_or(end, |next| next.min(end));
    (halves.is_multiple_of(2), Some(next))
}

/// The width of the caret of `settings` at `dpi`, rounded, and at least one
/// pixel.
pub fn thickness(settings: CaretSettings, dpi: u32) -> u32 {
    let scaled = (settings.width.saturating_mul(dpi) + BASE_DPI / 2) / BASE_DPI;
    scaled.max(1)
}
