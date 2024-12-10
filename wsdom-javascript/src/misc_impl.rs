use std::future::IntoFuture;

use crate::{Promise, PromiseLike};

use super::Array;
use wsdom_core::{r#await::Await, Cast, JsCast, ToJs};

impl<'a, T, U, const N: usize> ToJs<Array<T>> for [&'a U; N]
where
    T: JsCast,
    U: ToJs<T>,
{
}
macro_rules! promise_like {
    ($pr:ident) => {
        impl<T: JsCast> IntoFuture for $pr<T>{
            type Output = T;
        
            type IntoFuture = Cast<Await,T>;
        
            fn into_future(self) -> Self::IntoFuture {
                Cast{
                    value: self.0.into_future(),
                    phantom: std::marker::PhantomData,
                }
            }
        }
    };
}
promise_like!(PromiseLike);
promise_like!(Promise);