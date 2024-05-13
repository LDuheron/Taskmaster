pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // -- parser
    FieldNumProcsIsNotPositiveNumber { str: String },
    FieldCommandIsEmpty,
    FieldCommandIsNotSet,
    // -- to others errors
    Default(String),
}

impl<T> From<T> for Error
where
    T: std::fmt::Display,
{
    fn from(value: T) -> Self {
        Self::Default(value.to_string())
    }
}
