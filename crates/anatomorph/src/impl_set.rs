use std::fmt::Debug;

pub trait InSet<T> {
    fn get(pools: &T) -> &Self
    where
        Self: Sized;
    fn get_mut(pools: &mut T) -> &mut Self
    where
        Self: Sized;
}

#[macro_export]
macro_rules! gen_set {
    ($(#[$outer:meta])* $vis:vis $name:ident:$traverser:path,$traverser_mut:path {$($field:ident : $type:ty),*}) => {
        $(#[$outer])*
        $vis struct $name{
            $(pub $field:$type,)*
        }

        $(
            impl $crate::impl_set::InSet<$name> for $type{
                fn get(set:&$name)->&Self where Self: Sized{
                    &set.$field
                }
                fn get_mut(set:&mut $name)->&mut Self where Self: Sized{
                    &mut set.$field
                }
            }
        )*

        impl $name{
            pub fn get<T:$crate::impl_set::InSet<Self>>(&self)->&T{
                T::get(self)
            }

            pub fn get_mut<T:$crate::impl_set::InSet<Self>>(&mut self)->&mut T{
                T::get_mut(self)
            }

            pub fn traverse<T:$traverser>(&self,traverser:&mut T){
                $(
                    traverser.run(&self.$field);
                )*
            }

            pub fn traverse_mut<T:$traverser_mut>(&mut self,traverser_mut:&mut T){
                $(
                    traverser_mut.run(&mut self.$field);
                )*
            }
        }
    };
}

#[test]
fn test() {
    trait VecsTraverser {
        fn run<T: Debug>(&mut self, v: &T);
    }
    trait VecsTraverserMut {
        fn run<T: Debug>(&mut self, v: &mut T);
    }
    gen_set!(#[derive(Debug,Default)] pub Vecs:VecsTraverser,VecsTraverserMut  { a: usize });
    struct A;
    impl VecsTraverser for A {
        fn run<T: Debug>(&mut self, v: &T) {
            println!("{v:?}")
        }
    }
    fn a<T: InSet<Vecs>>(vecs: Vecs) {
        vecs.get::<T>();
        vecs.traverse(&mut A);
    }
}
