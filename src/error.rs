pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    // -- parser
    CantLoadFile(String),
    NoJobEntry,
    CantParseEntry {
        entry_name: String,
        e: String,
    },
    CantParseField {
        field_name: String,
        value: String,
        type_name: String,
    },
    FieldBadFormat {
        field_name: String,
        msg: String,
    },
    FieldCommandIsNotSet,
    // -- to others errors
    Default(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
