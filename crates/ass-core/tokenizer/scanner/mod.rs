//! Token scanning methods for ASS tokenizer
//!
//! Provides specialized scanning functions for different ASS script elements
//! including section headers, style overrides, comments, and text content.

mod navigator;
mod text_scanner;
mod token_scanner;

#[cfg(test)]
mod tests1;
#[cfg(test)]
mod tests2;
#[cfg(test)]
mod tests3;
#[cfg(test)]
mod tests4;
#[cfg(test)]
mod tests5;
#[cfg(test)]
mod tests6;
#[cfg(test)]
mod tests7;
#[cfg(test)]
mod tests8;
#[cfg(test)]
mod tests9;

pub use navigator::CharNavigator;
pub use token_scanner::TokenScanner;
