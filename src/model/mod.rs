mod request;
mod response;

/// "pub use" is to re-export items from a module so that they are accessible from
/// other modules and allows access to the modules like "code_reader_core::model::THING"
/// and is done to allow for cleaner code for the reference and use of these items

pub use request::{ReadRequest, ReaderInput};
pub use response::ReaderOutput;