//! Pure locator bounds against supplied bytes or numeric metadata. These checks
//! do not admit a representation, decoder/profile, digest or current authority.

use std::{error::Error, fmt};

pub const MAX_BYTE_OFFSET: u64 = 9_007_199_254_740_991;
pub const MAX_PIXEL_COMPONENT: u32 = 2_147_483_647;
pub const MAX_AUDIO_END_MS: u32 = 300_000;

/// A rejected locator, without retaining source bytes or invalid input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidLocator;

impl fmt::Display for InvalidLocator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("invalid representation locator")
    }
}

impl Error for InvalidLocator {}

/// An immutable view borrowed from exact valid UTF-8 bytes, with byte offsets.
#[derive(Clone, Copy)]
pub struct Utf8Selection<'a> {
    start_byte: u64,
    end_byte: u64,
    selected: &'a str,
}

impl<'a> Utf8Selection<'a> {
    pub fn new(original: &'a [u8], start: i64, end: i64) -> Result<Self, InvalidLocator> {
        if start < 0 || start >= end || end > MAX_BYTE_OFFSET as i64 {
            return Err(InvalidLocator);
        }
        let start_index = usize::try_from(start).map_err(|_| InvalidLocator)?;
        let end_index = usize::try_from(end).map_err(|_| InvalidLocator)?;
        let text = std::str::from_utf8(original).map_err(|_| InvalidLocator)?;
        let selected = text.get(start_index..end_index).ok_or(InvalidLocator)?;
        Ok(Self {
            start_byte: start as u64,
            end_byte: end as u64,
            selected,
        })
    }

    pub const fn byte_range(self) -> (u64, u64) {
        (self.start_byte, self.end_byte)
    }

    pub const fn selected(self) -> &'a str {
        self.selected
    }
}

impl fmt::Debug for Utf8Selection<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug never writes the borrowed text, including valid protected text.
        formatter
            .debug_struct("Utf8Selection")
            .field("start_byte", &self.start_byte)
            .field("end_byte", &self.end_byte)
            .finish_non_exhaustive()
    }
}

/// The eight Exif transforms of pixel edges, including mirrors.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum Orientation {
    Normal = 1,
    MirrorHorizontal = 2,
    Rotate180 = 3,
    MirrorVertical = 4,
    Transpose = 5,
    Rotate90 = 6,
    Transverse = 7,
    Rotate270 = 8,
}

impl Orientation {
    pub fn from_exif(value: i64) -> Result<Self, InvalidLocator> {
        Ok(match value {
            1 => Self::Normal,
            2 => Self::MirrorHorizontal,
            3 => Self::Rotate180,
            4 => Self::MirrorVertical,
            5 => Self::Transpose,
            6 => Self::Rotate90,
            7 => Self::Transverse,
            8 => Self::Rotate270,
            _ => return Err(InvalidLocator),
        })
    }

    pub const fn exif(self) -> u8 {
        self as u8
    }

    pub fn upright_dimensions(self, [width, height]: [u32; 2]) -> Result<[u32; 2], InvalidLocator> {
        if width == 0 || height == 0 {
            return Err(InvalidLocator);
        }
        Ok(match self {
            Self::Transpose | Self::Rotate90 | Self::Transverse | Self::Rotate270 => {
                [height, width]
            }
            _ => [width, height],
        })
    }

    /// Map an inclusive edge coordinate. This is not a pixel-index transform.
    pub fn map_edge(
        self,
        dimensions: [u32; 2],
        [x, y]: [u32; 2],
    ) -> Result<[u32; 2], InvalidLocator> {
        self.upright_dimensions(dimensions)?;
        let [width, height] = dimensions;
        if x > width || y > height {
            return Err(InvalidLocator);
        }
        // Checked inclusive bounds prove each subtraction remains in u32.
        Ok(match self {
            Self::Normal => [x, y],
            Self::MirrorHorizontal => [width - x, y],
            Self::Rotate180 => [width - x, height - y],
            Self::MirrorVertical => [x, height - y],
            Self::Transpose => [y, x],
            Self::Rotate90 => [height - y, x],
            Self::Transverse => [height - y, width - x],
            Self::Rotate270 => [y, width - x],
        })
    }
}

/// Supplied numeric render bounds, without a source identity or profile claim.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PageBounds {
    pub page_count: u32,
    pub width: u32,
    pub height: u32,
}

fn physical_rectangle(
    bounds: PageBounds,
    page_index: i64,
    [x, y, width, height]: [i64; 4],
) -> Result<(u16, [u32; 4]), InvalidLocator> {
    let maximum = i64::from(u32::MAX);
    if bounds.width == 0
        || bounds.height == 0
        || !(0..=65535).contains(&page_index)
        || page_index >= i64::from(bounds.page_count)
        || !(0..=maximum).contains(&x)
        || !(0..=maximum).contains(&y)
        || !(1..=maximum).contains(&width)
        || !(1..=maximum).contains(&height)
    {
        return Err(InvalidLocator);
    }
    // Physical components are bounded before conversion. Wide sums cannot wrap.
    let right = (x as u64).checked_add(width as u64).ok_or(InvalidLocator)?;
    let bottom = (y as u64)
        .checked_add(height as u64)
        .ok_or(InvalidLocator)?;
    if right > u64::from(bounds.width) || bottom > u64::from(bounds.height) {
        return Err(InvalidLocator);
    }
    Ok((
        page_index as u16,
        [x as u32, y as u32, width as u32, height as u32],
    ))
}

/// A checked upright region against the supplied numeric context only.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PageRegion {
    bounds: PageBounds,
    page_index: u16,
    orientation: Orientation,
    coordinates: [u32; 4],
}

impl PageRegion {
    /// Check already-upright coordinates without applying the orientation again.
    pub fn from_upright(
        bounds: PageBounds,
        page_index: i64,
        orientation: Orientation,
        coordinates: [i64; 4],
    ) -> Result<Self, InvalidLocator> {
        let (page_index, coordinates) = physical_rectangle(bounds, page_index, coordinates)?;
        if coordinates.iter().any(|value| *value > MAX_PIXEL_COMPONENT) {
            return Err(InvalidLocator);
        }
        Ok(Self {
            bounds,
            page_index,
            orientation,
            coordinates,
        })
    }

    /// Check physical raw bounds, map all four corners once, then check wire capacity.
    pub fn from_raw(
        bounds: PageBounds,
        page_index: i64,
        orientation: Orientation,
        coordinates: [i64; 4],
    ) -> Result<Self, InvalidLocator> {
        let (_, [x, y, width, height]) = physical_rectangle(bounds, page_index, coordinates)?;
        let right = x.checked_add(width).ok_or(InvalidLocator)?;
        let bottom = y.checked_add(height).ok_or(InvalidLocator)?;
        let mut minimum = [u32::MAX; 2];
        let mut maximum = [0; 2];
        for corner in [[x, y], [right, y], [x, bottom], [right, bottom]] {
            let point = orientation.map_edge([bounds.width, bounds.height], corner)?;
            for axis in 0..2 {
                minimum[axis] = minimum[axis].min(point[axis]);
                maximum[axis] = maximum[axis].max(point[axis]);
            }
        }
        let [width, height] = orientation.upright_dimensions([bounds.width, bounds.height])?;
        let upright = PageBounds {
            page_count: bounds.page_count,
            width,
            height,
        };
        Self::from_upright(
            upright,
            page_index,
            orientation,
            [
                i64::from(minimum[0]),
                i64::from(minimum[1]),
                i64::from(maximum[0].checked_sub(minimum[0]).ok_or(InvalidLocator)?),
                i64::from(maximum[1].checked_sub(minimum[1]).ok_or(InvalidLocator)?),
            ],
        )
    }

    pub const fn bounds(self) -> PageBounds {
        self.bounds
    }

    pub const fn page_index(self) -> u16 {
        self.page_index
    }

    pub const fn source_orientation(self) -> Orientation {
        self.orientation
    }

    pub const fn coordinates(self) -> [u32; 4] {
        self.coordinates
    }

    pub const fn coordinate_space(self) -> &'static str {
        "upright_pixels_v1"
    }
}

/// Supplied audio metadata, without decoding, sample mapping or source admission.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioBounds {
    pub duration_ms: u32,
    pub channels: u32,
}

/// A checked half-open integer-millisecond interval in one supplied channel.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioSpan {
    bounds: AudioBounds,
    channel_index: u16,
    start_ms: u32,
    end_ms: u32,
}

impl AudioSpan {
    pub fn new(
        bounds: AudioBounds,
        channel_index: i64,
        start: i64,
        end: i64,
    ) -> Result<Self, InvalidLocator> {
        if !(0..=65535).contains(&channel_index)
            || channel_index >= i64::from(bounds.channels)
            || !(0..i64::from(MAX_AUDIO_END_MS)).contains(&start)
            || !(1..=i64::from(MAX_AUDIO_END_MS)).contains(&end)
            || start >= end
            || end > i64::from(bounds.duration_ms)
        {
            return Err(InvalidLocator);
        }
        Ok(Self {
            bounds,
            channel_index: channel_index as u16,
            start_ms: start as u32,
            end_ms: end as u32,
        })
    }

    pub const fn bounds(self) -> AudioBounds {
        self.bounds
    }

    pub const fn channel_index(self) -> u16 {
        self.channel_index
    }

    pub const fn millisecond_range(self) -> (u32, u32) {
        (self.start_ms, self.end_ms)
    }

    pub const fn duration_ms(self) -> u32 {
        // Construction proves the half-open interval is nonempty and ordered.
        self.end_ms - self.start_ms
    }
}
