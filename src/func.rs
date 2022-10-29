use bevy::prelude::Component;
use macros::impl_with_tuples;

#[derive(Component)]
pub struct Action<F>(F);

impl<F> Action<F> {
    pub fn new(f: F) -> Self {
        Action(f)
    }
}

macro_rules! impl_action {
    ($($param: ident),*) => {
        impl<$($param),*> Action<fn($($param),*)>{
        #[inline]
        #[allow(dead_code)]
            pub fn run(&self$(, $param: $param)*){
                self.0($($param),*)
            }
        }
    };
}

impl_with_tuples!(impl_action, 0, 16, P);
