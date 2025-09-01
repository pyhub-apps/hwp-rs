/// Tag IDs for DocInfo section records
pub mod doc_info {
    pub const DOCUMENT_PROPERTIES: u16 = 0x0010;
    pub const ID_MAPPINGS: u16 = 0x0011;
    pub const BIN_DATA: u16 = 0x0012;
    pub const FACE_NAME: u16 = 0x0013;
    pub const BORDER_FILL: u16 = 0x0014;
    pub const CHAR_SHAPE: u16 = 0x0015;
    pub const TAB_DEF: u16 = 0x0016;
    pub const NUMBERING: u16 = 0x0017;
    pub const BULLET: u16 = 0x0018;
    pub const PARA_SHAPE: u16 = 0x0019;
    pub const STYLE: u16 = 0x001A;
    pub const DOC_DATA: u16 = 0x001B;
    pub const DISTRIBUTE_DOC_DATA: u16 = 0x001C;
    pub const COMPATIBLE_DOCUMENT: u16 = 0x0020;
    pub const LAYOUT_COMPATIBILITY: u16 = 0x0021;
    pub const TRACK_CHANGE: u16 = 0x0022;
    pub const MEMO_SHAPE: u16 = 0x004C;
    pub const FORBIDDEN_CHAR: u16 = 0x004E;
    pub const TRACK_CHANGE_AUTHOR: u16 = 0x0050;
    pub const CHANGE_TRACKING: u16 = 0x00F0;
}

/// Tag IDs for Section records
pub mod section {
    pub const PARA_HEADER: u16 = 0x0050;
    pub const PARA_TEXT: u16 = 0x0051;
    pub const PARA_CHAR_SHAPE: u16 = 0x0052;
    pub const PARA_LINE_SEG: u16 = 0x0053;
    pub const PARA_RANGE_TAG: u16 = 0x0054;
    pub const CTRL_HEADER: u16 = 0x0055;
    pub const LIST_HEADER: u16 = 0x0056;
    pub const PAGE_DEF: u16 = 0x0057;
    pub const FOOTNOTE_SHAPE: u16 = 0x0058;
    pub const PAGE_BORDER_FILL: u16 = 0x0059;
    pub const SHAPE_COMPONENT: u16 = 0x005A;
    pub const TABLE: u16 = 0x005B;
    pub const SHAPE_COMPONENT_LINE: u16 = 0x005C;
    pub const SHAPE_COMPONENT_RECTANGLE: u16 = 0x005D;
    pub const SHAPE_COMPONENT_ELLIPSE: u16 = 0x005E;
    pub const SHAPE_COMPONENT_ARC: u16 = 0x005F;
    pub const SHAPE_COMPONENT_POLYGON: u16 = 0x0060;
    pub const SHAPE_COMPONENT_CURVE: u16 = 0x0061;
    pub const SHAPE_COMPONENT_OLE: u16 = 0x0062;
    pub const SHAPE_COMPONENT_PICTURE: u16 = 0x0063;
    pub const SHAPE_COMPONENT_CONTAINER: u16 = 0x0064;
    pub const CTRL_DATA: u16 = 0x0065;
    pub const EQEDIT: u16 = 0x0066;
    pub const SHAPE_COMPONENT_TEXTART: u16 = 0x0068;
    pub const FORM_OBJECT: u16 = 0x0069;
    pub const MEMO_LIST: u16 = 0x006A;
    pub const CHART_DATA: u16 = 0x006B;
    pub const VIDEO_DATA: u16 = 0x006C;
    pub const SHAPE_COMPONENT_UNKNOWN: u16 = 0x006D;
}