/// Fill types for shapes and backgrounds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FillType {
    None = 0,
    Solid = 1,
    Gradient = 2,
    Image = 3,
    Pattern = 4,
}

/// Gradient types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GradientType {
    Linear = 0,
    Radial = 1,
    Conical = 2,
    Square = 3,
}

/// Pattern types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PatternType {
    Horizontal = 0,
    Vertical = 1,
    BackSlash = 2,
    Slash = 3,
    Cross = 4,
    CrossDiagonal = 5,
}

/// Image fill mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFillMode {
    Tile = 0,
    TileHorizontal = 1,
    TileVertical = 2,
    Fit = 3,
    Center = 4,
    CenterTop = 5,
    CenterBottom = 6,
    LeftCenter = 7,
    LeftTop = 8,
    LeftBottom = 9,
    RightCenter = 10,
    RightTop = 11,
    RightBottom = 12,
    Zoom = 13,
}