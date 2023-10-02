//! Error handling of lboot.
use uefi::Error as UefiError;

/// An lboot-related error, some of which is converted from
/// a UEFI-related error.
#[derive(Debug)]
pub enum LbootError {
    /// Cannot open and load configuration file
    CannotOpenConfig,
    /// Configuration file syntax error
    WrongConfig,
    /// Boot entry error
    WrongEntry,
    /// Cannot generate correct image path
    GenerateImagePathError,
    /// Cannot load kernel image into memory
    CannotLoadImageIntoMemory,
    /// Pointer conversion error
    PointerConversionError,
    /// Other UEFI-related errors
    UefiError(UefiError),
}

impl From<uefi::Error> for LbootError {
    fn from(uefi_error: UefiError) -> Self {
        LbootError::UefiError(uefi_error)
    }
}

/// Return type of most lboot functions.
pub type Result<T> = core::result::Result<T, LbootError>;
