use super::ExcMut;
use crate::{Exc, ExchangeError};
use tower::{util::MapErr, Service};

/// Request and Response binding.
pub trait Request: Sized {
    /// Response type.
    type Response;
}

/// An adaptor for request.
pub trait Adaptor<R: Request>: Request {
    /// Convert from request.
    fn from_request(req: R) -> Result<Self, ExchangeError>;

    /// Convert into response.
    fn into_response(resp: Self::Response) -> Result<R::Response, ExchangeError>;
}

impl<T, R, E> Adaptor<R> for T
where
    T: Request,
    R: Request,
    T: TryFrom<R, Error = E>,
    T::Response: TryInto<R::Response, Error = E>,
    ExchangeError: From<E>,
{
    fn from_request(req: R) -> Result<Self, ExchangeError>
    where
        Self: Sized,
    {
        Ok(Self::try_from(req)?)
    }

    fn into_response(resp: Self::Response) -> Result<<R as Request>::Response, ExchangeError> {
        Ok(resp.try_into()?)
    }
}

/// An alias of [`Service`] with the required bounds.
pub trait ExcService<R>: Service<R, Response = R::Response, Error = ExchangeError>
where
    R: Request,
{
    /// Create a mutable reference of itself.
    fn as_service_mut(&mut self) -> ExcMut<'_, Self> {
        ExcMut { inner: self }
    }
}

impl<S, R> ExcService<R> for S
where
    S: Service<R, Response = R::Response, Error = ExchangeError>,
    R: Request,
{
}

type MapErrFn<E> = fn(E) -> ExchangeError;

/// Service that can be converted into a [`Exc`].
pub trait IntoExc<R>: Service<R, Response = R::Response>
where
    Self::Error: Into<ExchangeError>,
    R: Request,
{
    /// Convert into a [`Exc`].
    fn into_exc(self) -> Exc<MapErr<Self, MapErrFn<Self::Error>>, R>
    where
        Self: Sized,
    {
        Exc::new(MapErr::new(self, Self::Error::into))
    }
}

impl<S, R> IntoExc<R> for S
where
    S: Service<R, Response = R::Response>,
    S::Error: Into<ExchangeError>,
    R: Request,
{
}
