mod standard_library_en;

#[cfg(feature = "localization")]
mod standard_library_es;

#[cfg(feature = "localization")]
mod standard_library_de;

#[cfg(feature = "localization")]
mod standard_library_cat;

pub use standard_library_en::define_standard_library_en;

#[cfg(feature = "localization")]
pub use standard_library_es::define_standard_library_es;

#[cfg(feature = "localization")]
pub use standard_library_cat::define_standard_library_cat;

#[cfg(feature = "localization")]
pub use standard_library_de::define_standard_library_de;
