//! Colours, and the palette of a terminal.

/// A colour with 8 bits per channel and no transparency.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// The colour `amount` of the way from this one to `other`: 0 is this
    /// one, and 255 is `other`.
    pub const fn mix(self, other: Rgb, amount: u8) -> Rgb {
        const fn channel(from: u8, to: u8, amount: u8) -> u8 {
            let (from, to, amount) = (from as u32, to as u32, amount as u32);
            // Rounded to the nearest. The result is between `from` and `to`,
            // so it fits in a byte.
            ((from * (255 - amount) + to * amount + 127) / 255) as u8
        }
        Rgb::new(
            channel(self.r, other.r, amount),
            channel(self.g, other.g, amount),
            channel(self.b, other.b, amount),
        )
    }
}

/// The 16 colours of Campbell, the scheme of Windows Terminal by default and
/// of the console of Windows since Windows 10: black, red, green, yellow,
/// blue, magenta, cyan and white, and their bright versions.
pub const CAMPBELL: [Rgb; 16] = [
    Rgb::new(0x0c, 0x0c, 0x0c),
    Rgb::new(0xc5, 0x0f, 0x1f),
    Rgb::new(0x13, 0xa1, 0x0e),
    Rgb::new(0xc1, 0x9c, 0x00),
    Rgb::new(0x00, 0x37, 0xda),
    Rgb::new(0x88, 0x17, 0x98),
    Rgb::new(0x3a, 0x96, 0xdd),
    Rgb::new(0xcc, 0xcc, 0xcc),
    Rgb::new(0x76, 0x76, 0x76),
    Rgb::new(0xe7, 0x48, 0x56),
    Rgb::new(0x16, 0xc6, 0x0c),
    Rgb::new(0xf9, 0xf1, 0xa5),
    Rgb::new(0x3b, 0x78, 0xff),
    Rgb::new(0xb4, 0x00, 0x9e),
    Rgb::new(0x61, 0xd6, 0xd6),
    Rgb::new(0xf2, 0xf2, 0xf2),
];

/// The colour of the text of meiga.
pub const FOREGROUND: Rgb = Rgb::new(0xd4, 0xd7, 0xdd);
/// The colour of the background of meiga.
pub const BACKGROUND: Rgb = Rgb::new(0x16, 0x18, 0x1d);
/// The colour of the background of the selected text: the one of the dark
/// theme of Visual Studio Code. The text keeps its colour.
pub const SELECTION: Rgb = Rgb::new(0x26, 0x4f, 0x78);

/// The levels of each component in the cube of the 256 colours of xterm.
const CUBE_LEVELS: [u8; 6] = [0x00, 0x5f, 0x87, 0xaf, 0xd7, 0xff];

/// The colours of a terminal: the 256 that the programs choose by number,
/// the ones that the text, the background and the cursor have when the
/// program does not choose, and the one of the background of the selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    /// 0–15 are the named colours, 16–231 a cube of 6 × 6 × 6 and 232–255
    /// greys, from dark to light.
    pub colors: [Rgb; 256],
    pub foreground: Rgb,
    pub background: Rgb,
    pub cursor: Rgb,
    /// The background of the selected cells. The programs cannot change it.
    pub selection: Rgb,
}

impl Palette {
    /// The colours of meiga until it has themes (subphase 8.3): the 16 of
    /// Campbell, with the text and the background of meiga. The cursor has
    /// the colour of the text.
    pub const MEIGA: Palette =
        Palette::new(CAMPBELL, FOREGROUND, BACKGROUND, FOREGROUND, SELECTION);

    /// The palette of xterm with the 16 colours `named`.
    pub const fn new(
        named: [Rgb; 16],
        foreground: Rgb,
        background: Rgb,
        cursor: Rgb,
        selection: Rgb,
    ) -> Self {
        let mut colors = [Rgb::new(0, 0, 0); 256];
        let mut index = 0;
        while index < 256 {
            colors[index] = match index {
                0..16 => named[index],
                16..232 => {
                    let cube = index - 16;
                    Rgb::new(
                        CUBE_LEVELS[cube / 36],
                        CUBE_LEVELS[cube / 6 % 6],
                        CUBE_LEVELS[cube % 6],
                    )
                }
                // 8, 18… 238: less than 256.
                _ => {
                    let grey = (8 + 10 * (index - 232)) as u8;
                    Rgb::new(grey, grey, grey)
                }
            };
            index += 1;
        }
        Self {
            colors,
            foreground,
            background,
            cursor,
            selection,
        }
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::MEIGA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_palette_has_the_256_colours_of_xterm() {
        let palette = Palette::MEIGA;
        assert_eq!(palette.colors[..16], CAMPBELL);
        // The corners of the cube, and a colour inside it: 16 + 36 × 1 +
        // 6 × 2 + 3.
        assert_eq!(palette.colors[16], Rgb::new(0, 0, 0));
        assert_eq!(palette.colors[67], Rgb::new(0x5f, 0x87, 0xaf));
        assert_eq!(palette.colors[231], Rgb::new(0xff, 0xff, 0xff));
        assert_eq!(palette.colors[232], Rgb::new(8, 8, 8));
        assert_eq!(palette.colors[255], Rgb::new(238, 238, 238));
        assert_eq!(
            (
                palette.foreground,
                palette.background,
                palette.cursor,
                palette.selection
            ),
            (FOREGROUND, BACKGROUND, FOREGROUND, SELECTION)
        );
        assert_eq!(Palette::default(), palette);
    }

    #[test]
    fn mixing_goes_from_one_colour_to_the_other() {
        let (black, white) = (Rgb::new(0, 0, 0), Rgb::new(255, 255, 255));
        assert_eq!(black.mix(white, 0), black);
        assert_eq!(black.mix(white, 255), white);
        assert_eq!(black.mix(white, 85), Rgb::new(85, 85, 85));
        assert_eq!(white.mix(black, 85), Rgb::new(170, 170, 170));
        assert_eq!(
            Rgb::new(10, 200, 30).mix(Rgb::new(20, 100, 30), 128),
            Rgb::new(15, 150, 30)
        );
    }
}
