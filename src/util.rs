use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use futures::stream::{self, TryStreamExt};
use itertools::{EitherOrBoth, Itertools};

// this macro is used to easily clone objects for use in a closure
// supports mutable variable bindings
macro_rules! enclose {
    ( __priv (mut $x:ident) ) => {
        let mut $x = $x.clone();
    };
    ( __priv ($x:ident) ) => {
        let $x = $x.clone();
    };
    ( ($( $($x:ident)+ ),* $(,)?) $y:expr ) => {
        {
            $(enclose! { __priv ( $($x)+ ) };)*
            $y
        }
    };
}
pub(crate) use enclose;

// in case you were wondering why this app takes to long to compile...
// this is the reason!
//
// I am lazy and don't want to get bothered by generic closure type parameters throughout my code,
// so I hack them away by moving the function definition into an async trait. The concrete
// implementation of the AdjustingVecHandlers still includes the closure types. Inside the AdjustingVec
// I am only using the trait, where the closure types get lost. Therefore I don't need the closure
// types when creating a AdjustingVec. The compilation takes so long because the compiler is optimizing
// the dyn AdjustingVecHandlers in the AdjustingVec into their concrete types, leaving one type
// for each instance of the AdjustingVec... So, basically the compiler is taking care of my laziness.
#[async_trait(?Send)]
trait AdjustingVecHandlers<T, E> {
    async fn create(&self) -> Result<T, E>
    where
        T: 'async_trait,
        E: 'async_trait;
    async fn destroy(&self, item: T) -> Result<(), E>
    where
        T: 'async_trait,
        E: 'async_trait;
}

struct AdjustingVecHandlersImpl<C, D> {
    creator: C,
    destroyer: D,
}

#[async_trait(?Send)]
impl<T, E, C, D> AdjustingVecHandlers<T, E> for AdjustingVecHandlersImpl<C, D>
where
    C: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>>>>,
    D: Fn(T) -> Pin<Box<dyn Future<Output = Result<(), E>>>>,
{
    async fn create(&self) -> Result<T, E>
    where
        T: 'async_trait,
        E: 'async_trait,
    {
        (self.creator)().await
    }

    async fn destroy(&self, item: T) -> Result<(), E>
    where
        T: 'async_trait,
        E: 'async_trait,
    {
        (&self.destroyer)(item).await
    }
}

pub struct AdjustingVec<T, E> {
    data: Vec<T>,
    handlers: Box<dyn AdjustingVecHandlers<T, E>>,
}

impl<T: std::fmt::Debug, E> std::fmt::Debug for AdjustingVec<T, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_list().entries(&self.data).finish()
    }
}

impl<T, E> AdjustingVec<T, E> {
    #[allow(clippy::as_conversions)]
    pub fn new<C, FC, D, FD>(creator: C, destroyer: D) -> Self
    where
        C: 'static + Fn() -> FC,
        FC: 'static + Future<Output = Result<T, E>>,
        D: 'static + Fn(T) -> FD,
        FD: 'static + Future<Output = Result<(), E>>,
    {
        Self {
            data: Vec::new(),
            handlers: Box::new(AdjustingVecHandlersImpl {
                // create pinned boxes for the futures returned by creator and destroyer
                creator: move || Box::pin(creator()) as Pin<Box<dyn Future<Output = Result<T, E>>>>,
                destroyer: move |item| {
                    Box::pin(destroyer(item)) as Pin<Box<dyn Future<Output = Result<(), E>>>>
                },
            }),
        }
    }

    pub async fn adjust<A, I, F, FT>(&mut self, iterable: I, mapper: F) -> Result<(), E>
    where
        I: IntoIterator<Item = A>,
        F: Fn(T, A) -> FT,
        FT: Future<Output = Result<T, E>>,
    {
        let mut data: Vec<T> = self.data.drain(..).collect();
        let iter = data.drain(..).zip_longest(iterable.into_iter()).map(Ok);
        self.data = stream::iter(iter)
            .try_filter_map(|zipped| async {
                match zipped {
                    EitherOrBoth::Left(current) => {
                        self.handlers.destroy(current).await?;
                        Ok(None)
                    }
                    EitherOrBoth::Right(next) => {
                        Ok(Some(mapper(self.handlers.create().await?, next).await?))
                    }
                    EitherOrBoth::Both(current, next) => Ok(Some(mapper(current, next).await?)),
                }
            })
            .try_collect()
            .await?;

        Ok(())
    }
}
