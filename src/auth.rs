use crate::{cx::RouteContext, tokens::models::MavenToken};
use anyhow::{Result, anyhow};
use axum::http::{HeaderName, HeaderValue, header::AUTHORIZATION};
use axum_extra::headers::{
    Authorization, Error, Header,
    authorization::{Basic, Credentials},
};

#[derive(Clone, PartialEq, Debug)]
pub enum AnyAuth {
    Basic(Authorization<Basic>),
    None,
}

impl AnyAuth {
    pub fn encode(&self) -> HeaderValue {
        match self {
            Self::Basic(basic) => basic.0.encode(),
            Self::None => unreachable!(),
        }
    }

    pub fn scheme(&self) -> &'static str {
        match self {
            Self::Basic(_) => Basic::SCHEME,
            Self::None => unreachable!(),
        }
    }

    pub async fn get_token(&self, cx: &RouteContext) -> Result<MavenToken> {
        match self {
            Self::Basic(basic) => cx.get_token(basic.username(), basic.password()).await,
            Self::None => Err(anyhow!("A token is required!")),
        }
    }
}

impl Header for AnyAuth {
    fn name() -> &'static HeaderName {
        &AUTHORIZATION
    }

    fn decode<'i, I: Iterator<Item = &'i HeaderValue>>(values: &mut I) -> Result<Self, Error> {
        debug!("Decoding auth header...");

        values
            .next()
            .and_then(|val| {
                let slice = val.as_bytes();

                if slice.len() > Basic::SCHEME.len()
                    && slice[Basic::SCHEME.len()] == b' '
                    && slice[..Basic::SCHEME.len()].eq_ignore_ascii_case(Basic::SCHEME.as_bytes())
                {
                    Basic::decode(val).map(Authorization).map(AnyAuth::Basic)
                } else {
                    Some(Self::None)
                }
            })
            .ok_or_else(Error::invalid)
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        if *self == Self::None {
            return;
        }

        let mut value = self.encode();

        value.set_sensitive(true);

        debug_assert!(
            value.as_bytes().starts_with(self.scheme().as_bytes()),
            "Credentials::encode should include its scheme: scheme = {:?}, encoded = {:?}",
            self.scheme(),
            value,
        );

        values.extend(std::iter::once(value));
    }
}
