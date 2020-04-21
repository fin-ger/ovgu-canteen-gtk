use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use futures::stream::{self, TryStreamExt};
use itertools::{EitherOrBoth, Itertools};

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
