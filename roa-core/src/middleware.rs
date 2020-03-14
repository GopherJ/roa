use crate::{async_trait, Context, Result};
use std::future::Future;

/// ### Middleware
///
/// There are two kinds of middlewares,
/// the one is functional middlewares, the another is trait middlewares.
///
/// #### Functional Middlewares
///
/// A normal functional middleware is an object implements `Fn` trait:
///
/// ```rust
/// use roa_core::{Context, Next, Result, Middleware};
/// use std::future::Future;
///
/// fn is_middleware<S>(middleware: impl for<'a> Middleware<'a, S>) {
/// }
///
/// async fn middleware(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     Ok(())
/// }
///
/// is_middleware(middleware);
/// ```
///
/// Closures are also supported, but feature(async_closure) is required:
///
/// #### Trait Middlewares
///
/// A trait middleware is an object implementing trait `Middleware`.
///
/// ```rust
/// use roa_core::{Middleware, Context, Next, Result, async_trait};
/// use async_std::sync::Arc;
/// use std::time::Instant;
///
/// fn is_middleware<S>(middleware: impl for<'a> Middleware<'a, S>) {}
///
/// struct Logger;
///
/// #[async_trait(?Send)]
/// impl <'a> Middleware<'a, ()> for Logger {
///     async fn handle(&'a self, ctx: &'a mut Context<()>, next: Next<'a>) -> Result {
///         let start = Instant::now();
///         let result = next.await;
///         println!("time elapsed: {}ms", start.elapsed().as_millis());
///         result
///     }
/// }
///
/// is_middleware(Logger);
/// ```
#[async_trait(?Send)]
pub trait Middleware<'a, S>: 'static + Sync + Send {
    /// Handle context and next, then return a future to get status.
    async fn handle(&'a self, ctx: &'a mut Context<S>, next: Next<'a>) -> Result;
}

#[async_trait(?Send)]
impl<'a, S, T, F> Middleware<'a, S> for T
where
    S: 'a,
    T: 'static + Send + Sync + Fn(&'a mut Context<S>, Next<'a>) -> F,
    F: 'a + Future<Output = Result>,
{
    #[inline]
    async fn handle(&'a self, ctx: &'a mut Context<S>, next: Next<'a>) -> Result {
        (self)(ctx, next).await
    }
}

/// ### Endpoint
///
/// There are two kinds of endpoints,
/// the one is functional endpoints, the another is trait endpoints.
///
/// #### Functional Endpoints
///
/// A normal functional endpoint is an object implements `Fn` trait:
///
/// ```rust
/// use roa_core::{Context, Next, Result, Endpoint};
/// use std::future::Future;
///
/// fn is_endpoint<S>(endpoint: impl for<'a> Endpoint<'a, S>) {
/// }
///
/// async fn endpoint(ctx: &mut Context<()>) -> Result {
///     Ok(())
/// }
///
/// is_endpoint(endpoint);
/// ```
///
/// Closures are also supported, but feature(async_closure) is required:
///
/// #### Trait Endpoints
///
/// A trait endpoint is an object implementing trait `Endpoint`.
///
/// ```rust
/// use roa_core::{Endpoint, Context, Next, Result, async_trait};
/// use async_std::sync::Arc;
/// use std::time::Instant;
///
/// fn is_endpoint<S>(endpoint: impl for<'a> Endpoint<'a, S>) {
/// }
///
/// struct Logger;
///
/// #[async_trait(?Send)]
/// impl <'a> Endpoint<'a, ()> for Logger {
///     async fn call(&'a self, ctx: &'a mut Context<()>) -> Result {
///         Ok(())
///     }
/// }
///
/// is_endpoint(Logger);
/// ```
#[async_trait(?Send)]
pub trait Endpoint<'a, S>: 'static + Sync + Send {
    /// Call this endpoint.
    async fn call(&'a self, ctx: &'a mut Context<S>) -> Result;
}

#[async_trait(?Send)]
impl<'a, S, T, F> Endpoint<'a, S> for T
where
    S: 'a,
    T: 'static + Send + Sync + Fn(&'a mut Context<S>) -> F,
    F: 'a + Future<Output = Result>,
{
    #[inline]
    async fn call(&'a self, ctx: &'a mut Context<S>) -> Result {
        (self)(ctx).await
    }
}

/// Fake middleware.
#[async_trait(?Send)]
impl<'a, S> Middleware<'a, S> for () {
    #[inline]
    async fn handle(&'a self, _ctx: &'a mut Context<S>, next: Next<'a>) -> Result<()> {
        next.await
    }
}

/// Fake endpoint.
#[async_trait(?Send)]
impl<'a, S> Endpoint<'a, S> for () {
    #[inline]
    async fn call(&'a self, _ctx: &'a mut Context<S>) -> Result<()> {
        Ok(())
    }
}

/// Type of the second parameter in a middleware.
///
/// `Next` is usually a closure capturing the next middleware, context and the next `Next`.
///
/// Developer of middleware can jump to next middleware by calling `next.await`.
///
/// ### Example
///
/// ```rust
/// use roa_core::{App, Context, Result, Error, MiddlewareExt, Next};
/// use roa_core::http::StatusCode;
///
/// let app = App::new(())
///     .gate(first)
///     .gate(second)
///     .gate(third)
///     .end(end);
/// async fn first(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     assert!(ctx.store("id", "1").is_none());
///     next.await?;
///     assert_eq!("5", *ctx.load::<&'static str>("id").unwrap());
///     Ok(())
/// }
/// async fn second(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     assert_eq!("1", *ctx.load::<&'static str>("id").unwrap());
///     assert_eq!("1", *ctx.store("id", "2").unwrap());
///     next.await?;
///     assert_eq!("4", *ctx.store("id", "5").unwrap());
///     Ok(())
/// }
/// async fn third(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     assert_eq!("2", *ctx.store("id", "3").unwrap());
///     next.await?; // next is none; do nothing
///     assert_eq!("3", *ctx.store("id", "4").unwrap());
///     Ok(())
/// }
///
/// async fn end(ctx: &mut Context<()>) -> Result {
///     assert_eq!("3", *ctx.load::<&'static str>("id").unwrap());
///     Ok(())
/// }
/// ```
///
/// ### Error Handling
///
/// You can catch or straightly throw a Error returned by next.
///
/// ```rust
/// use roa_core::{App, Context, Result, Error, MiddlewareExt, Next, throw};
/// use roa_core::http::StatusCode;
///         
/// let app = App::new(())
///     .gate(catch)
///     .gate(gate)
///     .end(end);
///
/// async fn catch(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     // catch
///     if let Err(err) = next.await {
///         // teapot is ok
///         if err.status_code != StatusCode::IM_A_TEAPOT {
///             return Err(err)
///         }
///     }
///     Ok(())
/// }
/// async fn gate(ctx: &mut Context<()>, next: Next<'_>) -> Result {
///     next.await?; // just throw
///     unreachable!()
/// }
///
/// async fn end(ctx: &mut Context<()>) -> Result {
///     throw!(StatusCode::IM_A_TEAPOT, "I'm a teapot!")
/// }
/// ```
///
pub type Next<'a> = &'a mut (dyn Unpin + Future<Output = Result<()>>);
