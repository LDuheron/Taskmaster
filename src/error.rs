pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    Default(String),
    BadNumberOfArguments(String),
    CantLoadFile(String),
    NoJobEntry,
    JobEntryNameWithNonAlphanumChar,
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
    CantOpenLogFile(String),
    CommandIsNotSupported(String),
    IO(String),
    StartJobFail(String),
    StopJobFail(String),
    StatusJobFail(String),
    ParseClientInput(String),
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}
