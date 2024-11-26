use {
    hashbrown::{hash_map::Entry, HashMap},
    rustc_hash::FxBuildHasher,
    std::any::{Any, TypeId},
};

/// The resources that are available to the UI.
#[derive(Default)]
pub struct UiResources(HashMap<TypeId, Box<dyn Any>, FxBuildHasher>);

impl UiResources {
    /// Inserts a value into the resource map.
    ///
    /// # Returns
    ///
    /// If a value with the same type was already present in the map, it is returned.
    pub fn insert<T: Any>(&mut self, val: T) -> Option<T> {
        match self.0.entry(TypeId::of::<T>()) {
            Entry::Occupied(mut entry) => {
                let occupied = unsafe { entry.get_mut().downcast_mut::<T>().unwrap_unchecked() };
                Some(std::mem::replace(occupied, val))
            }
            Entry::Vacant(entry) => {
                entry.insert(Box::new(val));
                None
            }
        }
    }

    /// Inserts a value into the resource map if it is not already present.
    pub fn get_or_insert<T: Any>(&mut self, val: impl FnOnce() -> T) -> &mut T {
        unsafe {
            self.0
                .entry(TypeId::of::<T>())
                .or_insert_with(|| Box::new(val()))
                .downcast_mut()
                .unwrap_unchecked()
        }
    }

    /// Inserts a default value into the resource map if it is not already present.
    pub fn get_or_insert_default<T: Any + Default>(&mut self) -> &mut T {
        self.get_or_insert(Default::default)
    }

    /// Gets a reference to a value of type `T` from the resource map.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0
            .get(&TypeId::of::<T>())
            .map(|val| unsafe { val.downcast_ref().unwrap_unchecked() })
    }

    /// Gets a mutable reference to a value of type `T` from the resource map.
    pub fn get_mut<T: Any>(&mut self) -> Option<&mut T> {
        self.0
            .get_mut(&TypeId::of::<T>())
            .map(|val| unsafe { val.downcast_mut().unwrap_unchecked() })
    }

    /// Gets many references to values of types `T` from the resource map.
    #[inline]
    pub fn get_many_mut<T: _private::GetManyMut>(&mut self) -> T::Output<'_> {
        T::get_many_mut(self)
    }
}

mod _private {
    use {crate::UiResources, std::any::TypeId};

    pub trait GetManyMut {
        type Output<'a>;

        fn get_many_mut(res: &mut UiResources) -> Self::Output<'_>;
    }

    macro_rules! impl_get_many {
        ($($ty:ident),* $(,)?) => {
            impl<$($ty: 'static),*> GetManyMut for ($($ty,)*) {
                type Output<'a> = ($(Option<&'a mut $ty>,)*);

                #[allow(clippy::unused_unit, non_snake_case)]
                fn get_many_mut(res: &mut UiResources) -> Self::Output<'_> {
                    let [$($ty),*] = res.0.get_many_mut([$(&TypeId::of::<$ty>()),*]);
                    unsafe { ( $( $ty.map(|x| x.downcast_mut().unwrap_unchecked()), )* ) }
                }
            }
        };
    }

    impl_get_many!(A);
    impl_get_many!(A, B);
    impl_get_many!(A, B, C);
    impl_get_many!(A, B, C, D);
}
