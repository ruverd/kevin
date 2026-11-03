mod composer;
mod design;
mod memory;
mod model;
mod routing;
mod validation;

pub use composer::*;
pub use design::*;
pub use memory::*;
pub use model::*;
pub use routing::*;
pub use validation::*;

pub type Result<T> = std::result::Result<T, String>;
