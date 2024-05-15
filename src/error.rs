pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    // -- parser
    CantParseEntry {
        entry_name: String,
        e: String,
    },
    NoJobEntry,
    CantLoadFile(String),
    FieldNumProcsIsNotPositiveNumber {
        str: String,
    },
    FieldCommandIsEmpty,
    FieldCommandIsNotSet,
    CantParseField {
        field_name: String,
        value: String,
        type_name: String,
    },
    // -- to others errors
    Default(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}
