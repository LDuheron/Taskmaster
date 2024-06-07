pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    // -- parser
    BadNumberOfArguments(String),
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
    CantParseEnvEntry(String),
    IO(String),
    Default(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
