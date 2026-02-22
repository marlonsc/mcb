/// Generates `Arc`-cloning getter methods for DI container fields.
#[macro_export]
macro_rules! arc_getters {
    () => {};

    ($name:ident : $ty:ty => $field:ident $(= $impl:ty)? , $($rest:tt)*) => {
        #[doc = concat!("Get `", stringify!($name), "`.")]
        #[must_use]
        pub fn $name(&self) -> std::sync::Arc<$ty> {
            std::sync::Arc::clone(&self.$field)
        }

        $crate::arc_getters! { $($rest)* }
    };

    ($name:ident : $ty:ty => $field:ident $(= $impl:ty)? $(,)?) => {
        #[doc = concat!("Get `", stringify!($name), "`.")]
        #[must_use]
        pub fn $name(&self) -> std::sync::Arc<$ty> {
            std::sync::Arc::clone(&self.$field)
        }
    };

    ($name:ident : $ty:ty $(= $impl:ty)? , $($rest:tt)*) => {
        #[doc = concat!("Get `", stringify!($name), "`.")]
        #[must_use]
        pub fn $name(&self) -> std::sync::Arc<$ty> {
            std::sync::Arc::clone(&self.$name)
        }

        $crate::arc_getters! { $($rest)* }
    };

    ($name:ident : $ty:ty $(= $impl:ty)? $(,)?) => {
        #[doc = concat!("Get `", stringify!($name), "`.")]
        #[must_use]
        pub fn $name(&self) -> std::sync::Arc<$ty> {
            std::sync::Arc::clone(&self.$name)
        }
    };
}
