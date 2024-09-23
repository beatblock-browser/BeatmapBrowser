use std::pin::{pin, Pin};
use std::task::{Context, Poll};
use anyhow::Error;
use http_body_util::Full;
use hyper::body::{Body, Bytes, Frame};

pub enum EitherBody {
    StaticBody(hyper_staticfile::Body),
    Full(Full<Bytes>),
}

impl From<hyper_staticfile::Body> for EitherBody {
    fn from(base: hyper_staticfile::Body) -> Self {
        EitherBody::StaticBody(base)
    }
}

impl From<Full<Bytes>> for EitherBody {
    fn from(base: Full<Bytes>) -> Self {
        EitherBody::Full(base)
    }
}

impl Body for EitherBody {
    type Data = Bytes;
    type Error = Error;

    fn poll_frame(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match self.get_mut() {
            EitherBody::StaticBody(ref mut base) => {
                let mut pinned = pin!(base);
                pinned.as_mut().poll_frame(cx).map(|opt| opt.map(|res| res.map_err(Error::from)))
            },
            EitherBody::Full(ref mut base) => {
                let mut pinned = pin!(base);
                pinned.as_mut().poll_frame(cx).map(|opt| opt.map(|res| res.map_err(Error::from)))
            },
        }
    }
}