use std::any::type_name;

use crate::{Downcast, Key, SchemaError, SchemaExt, SchemaResult, Typed};

macro_rules! any_of_impl {
    ($($name:ident = $($v:ident($g:ident)),*;)*) => {$(
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub enum $name<$($g),*> {
            $(
                $v($g)
            ),*
        }

        impl<'a, NK, EK, S, T, $($g),*> Downcast<'a, NK, EK, $name<$(&'a $g),*>, S> for T
        where
            T: Typed $( + Downcast<'a, NK, EK, &'a $g, S>)*,
            NK: Key,
            EK: Key,
            S: SchemaExt<NK, EK>,
            $(
                $g: Typed
            ),*
        {
            fn downcast<'b: 'a>(&'a self) -> SchemaResult<$name<$(&'a $g),*>, NK, EK, S> {
                $(
                    let n1 = Downcast::<'a, NK, EK, &'a $g, S>::downcast(self);
    
                    if let Ok(n1) = n1 {
                        return Ok($name::$v(n1));
                    }

                )*

                let type_names = &[
                    $(
                        stringify!($g)
                    ),*
                ];

                Err(SchemaError::<NK, EK, S>::DownCastFailed(
                    format!("Either<{}>", type_names.join(", ")), 
                    self.get_type().to_string())
                )
            }
        }
    )*};
}

any_of_impl!{
    Either2 = One(T1), Two(T2);
    Either3 = One(T1), Two(T2), Three(T3);
    Either4 = One(T1), Two(T2), Three(T3), Four(T4);
    Either5 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5);
    Either6 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5), Six(T6);
    Either7 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5), Six(T6), Seven(T7);
    Either8 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5), Six(T6), Seven(T7), Eight(T8);
    Either9 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5), Six(T6), Seven(T7), Eight(T8), Nine(T9);
    Either10 = One(T1), Two(T2), Three(T3), Four(T4), Five(T5), Six(T6), Seven(T7), Eight(T8), Nine(T9), Ten(T10);
}