use std::{
    any::Any,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
};

pub struct DowncastBox<T: ?Sized> {
    general: Box<T>,
    any: ManuallyDrop<Box<dyn Any>>,
}

impl<T: ?Sized + Any> DowncastBox<T> {
    pub fn new<R: Sized + 'static>(value: Box<R>) -> Self
    where
        Box<R>: Into<Box<T>>,
    {
        let raw = Box::leak(value);
        let value_any: Box<R> = unsafe { Box::from_raw(raw) };
        let value_general: Box<R> = unsafe { Box::from_raw(raw) };
        let value_any: Box<dyn Any> = value_any;
        let value_general: Box<T> = value_general.into();
        Self {
            general: value_general,
            any: ManuallyDrop::new(value_any),
        }
    }

    #[allow(dead_code)]
    pub fn as_any(&self) -> &dyn Any {
        &**self.any
    }

    pub fn as_any_mut(&mut self) -> &mut dyn Any {
        &mut **self.any
    }
}

impl<T: ?Sized + Any> Deref for DowncastBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.general
    }
}

impl<T: ?Sized + Any> DerefMut for DowncastBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.general
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait T1: Any {}

    struct V1 {}

    impl From<Box<V1>> for Box<dyn T1> {
        fn from(val: Box<V1>) -> Self {
            val
        }
    }

    impl T1 for V1 {}

    #[test]
    fn test_downcast_box() {
        let mut v = DowncastBox::<dyn T1>::new::<V1>(Box::new(V1 {}));
        let _: &mut V1 = v.as_any_mut().downcast_mut().expect("invalid root type");
    }
}
