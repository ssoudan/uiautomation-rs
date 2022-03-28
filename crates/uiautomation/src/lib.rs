pub mod variants;
pub mod errors;
pub mod core;
pub mod patterns;
pub mod conditions;

pub use crate::errors::Error;
pub use crate::errors::Result;

pub use crate::core::UIAutomation;
pub use crate::core::UIElement;
pub use crate::core::UITreeWalker;
pub use crate::core::UIMatcher;