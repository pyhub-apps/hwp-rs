/// Control IDs for various control elements in HWP documents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum CtrlId {
    Table = 0x00746c62,           // 'tbl\0' in little-endian
    GenShapeObject = 0x6F736467,  // 'gso\0' in little-endian  
    Line = 0x006C696E,             // 'lin\0' in little-endian
    Rectangle = 0x00636572,        // 'rec\0' in little-endian
    Ellipse = 0x00636565,          // 'ell\0' in little-endian
    Arc = 0x00636561,              // 'arc\0' in little-endian
    Polygon = 0x006C6F70,          // 'pol\0' in little-endian
    Curve = 0x00727563,            // 'cur\0' in little-endian
    Equation = 0x00716571,         // 'eqe\0' in little-endian
    Picture = 0x00636970,          // 'pic\0' in little-endian
    Ole = 0x00656C6F,              // 'ole\0' in little-endian
    Container = 0x006E6F63,        // 'con\0' in little-endian
    Header = 0x00646568,           // 'hed\0' in little-endian
    Footer = 0x00746F66,           // 'fot\0' in little-endian
    PageNumPos = 0x00706E70,       // 'pnp\0' in little-endian
    NewNum = 0x006E776E,           // 'nwn\0' in little-endian
    Footnote = 0x00746E66,         // 'fnt\0' in little-endian
    Endnote = 0x00746E65,          // 'ent\0' in little-endian
    AutoNum = 0x006D756E,          // 'num\0' in little-endian
    PageHiding = 0x00646870,       // 'phd\0' in little-endian
    PageOddEvenAdjust = 0x61656F70, // 'poea' in little-endian
    PageBreak = 0x006B6270,        // 'pbk\0' in little-endian
    Field = 0x006C6466,            // 'fld\0' in little-endian
    Bookmark = 0x006B6D62,         // 'bmk\0' in little-endian
    DutmalTitle = 0x74747564,      // 'dutt' in little-endian
    IndexMark = 0x006B6469,        // 'idx\0' in little-endian
}

impl CtrlId {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x00746c62 => Some(Self::Table),
            0x6F736467 => Some(Self::GenShapeObject),
            0x006C696E => Some(Self::Line),
            0x00636572 => Some(Self::Rectangle),
            0x00636565 => Some(Self::Ellipse),
            0x00636561 => Some(Self::Arc),
            0x006C6F70 => Some(Self::Polygon),
            0x00727563 => Some(Self::Curve),
            0x00716571 => Some(Self::Equation),
            0x00636970 => Some(Self::Picture),
            0x00656C6F => Some(Self::Ole),
            0x006E6F63 => Some(Self::Container),
            0x00646568 => Some(Self::Header),
            0x00746F66 => Some(Self::Footer),
            0x006D756E => Some(Self::AutoNum),
            0x00706E70 => Some(Self::PageNumPos),
            0x006E776E => Some(Self::NewNum),
            0x00746E66 => Some(Self::Footnote),
            0x00746E65 => Some(Self::Endnote),
            0x00646870 => Some(Self::PageHiding),
            0x61656F70 => Some(Self::PageOddEvenAdjust),
            0x006B6270 => Some(Self::PageBreak),
            0x006C6466 => Some(Self::Field),
            0x006B6D62 => Some(Self::Bookmark),
            0x74747564 => Some(Self::DutmalTitle),
            0x006B6469 => Some(Self::IndexMark),
            _ => None,
        }
    }
}