use core::fmt;
use std::borrow::Borrow;

use anyhow::{Context as _, bail};
use reqwest::Url;
use smallstr::SmallString;
use url::UrlQuery;

type Buffer = SmallString<[u8; 128]>;

pub trait BuildUrl<C> {
    fn build_url(&self, url_builder: &mut UrlBuilder, context: &C) -> anyhow::Result<()>;
}

impl<C> BuildUrl<C> for str {
    fn build_url(&self, url_builder: &mut UrlBuilder, _context: &C) -> anyhow::Result<()> {
        url_builder.join_path(self)
    }
}

impl<C> BuildUrl<C> for [&str] {
    fn build_url(&self, url_builder: &mut UrlBuilder, _context: &C) -> anyhow::Result<()> {
        url_builder.add_path_segments(self)
    }
}

impl<C, const SIZE: usize> BuildUrl<C> for [&str; SIZE] {
    fn build_url(&self, url_builder: &mut UrlBuilder, _context: &C) -> anyhow::Result<()> {
        url_builder.add_path_segments(*self)
    }
}

pub struct UrlBuilder<'a> {
    res: UrlBuilderInner<'a>,
    buffer: Buffer,
}

impl<'a> UrlBuilder<'a> {
    pub(crate) fn build<C, B: ?Sized + BuildUrl<C>>(
        base_url: &'a Url,
        build: &B,
        context: &C,
    ) -> anyhow::Result<Url> {
        let mut builder = Self {
            res: UrlBuilderInner::Shared(base_url),
            buffer: SmallString::new(),
        };
        build.build_url(&mut builder, context)?;
        builder.res.take()
    }
}

#[derive(Default)]
enum UrlBuilderInner<'a> {
    Owned(Url),
    Shared(&'a Url),
    #[default]
    Invalid,
}

impl<'a> UrlBuilderInner<'a> {
    fn get_mut(&mut self) -> anyhow::Result<&mut Url> {
        match &self {
            UrlBuilderInner::Shared(url) => {
                *self = UrlBuilderInner::Owned((*url).clone());
            }
            UrlBuilderInner::Owned(_) => {}
            UrlBuilderInner::Invalid => bail!("url builder is in invalid state"),
        }
        if let UrlBuilderInner::Owned(url) = self {
            Ok(url)
        } else {
            unreachable!()
        }
    }

    fn take(&mut self) -> anyhow::Result<Url> {
        match std::mem::take(self) {
            UrlBuilderInner::Shared(url) => Ok(url.clone()),
            UrlBuilderInner::Owned(url) => Ok(url),
            UrlBuilderInner::Invalid => bail!("url builder is in invalid state"),
        }
    }

    fn get(&self) -> anyhow::Result<&Url> {
        match self {
            UrlBuilderInner::Shared(url) => Ok(url),
            UrlBuilderInner::Owned(url) => Ok(url),
            UrlBuilderInner::Invalid => bail!("url builder is in invalid state"),
        }
    }

    fn set(&mut self, url: Url) -> anyhow::Result<()> {
        *self = UrlBuilderInner::Owned(url);
        Ok(())
    }
}

impl<'a> UrlBuilder<'a> {
    pub fn join_path(&mut self, path: &str) -> anyhow::Result<()> {
        let url = self.res.get()?;
        let url = url.join(path).context("join base url and path")?;
        self.res.set(url)
    }

    pub fn add_path_segments<I>(&mut self, path_segments: I) -> anyhow::Result<()>
    where
        I: IntoIterator,
        I::Item: AsRef<str>,
    {
        let mut url = self.res.take()?;
        match url.path_segments_mut() {
            Ok(mut segments) => {
                segments.extend(path_segments);
            }
            Err(()) => {
                bail!("url can't be base");
            }
        };
        self.res.set(url)
    }

    pub fn query_builder(&mut self) -> anyhow::Result<UrlQueryBuilder> {
        let url = self.res.get_mut()?;
        Ok(UrlQueryBuilder {
            encoder: url.query_pairs_mut(),
            buffer: &mut self.buffer,
        })
    }

    pub fn add_query_pairs<I, K, V>(&mut self, query_pairs: I) -> anyhow::Result<()>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let mut url = self.res.take()?;
        url.query_pairs_mut().extend_pairs(query_pairs);
        self.res.set(url)
    }

    pub fn add_query_pairs_display<I, K, V>(&mut self, query_pairs: I) -> anyhow::Result<()>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: fmt::Display,
        V: fmt::Display,
    {
        let mut url = self.res.take()?;
        {
            let mut pairs = url.query_pairs_mut();
            for item in query_pairs.into_iter() {
                let (k, v) = item.borrow();
                self.buffer.clear();
                use fmt::Write;
                if writeln!(&mut self.buffer, "{k}").is_err() {
                    bail!("convert query key to string");
                }
                let k_end = self.buffer.len();
                if writeln!(&mut self.buffer, "{v}").is_err() {
                    bail!("convert query value to string");
                }
                let (k, v) = self.buffer.split_at(k_end);
                pairs.append_pair(k, v);
            }
        }
        self.res.set(url)
    }
}

pub struct UrlQueryBuilder<'a> {
    encoder: form_urlencoded::Serializer<'a, UrlQuery<'a>>,
    buffer: &'a mut Buffer,
}

impl<'a> UrlQueryBuilder<'a> {
    pub fn add_pair<K, V>(&mut self, name: K, value: V)
    where
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.encoder.append_pair(name.as_ref(), value.as_ref());
    }

    pub fn add_pairs<I, K, V>(&mut self, query_pairs: I)
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        self.encoder.extend_pairs(query_pairs);
    }

    pub fn display_pair<K: ?Sized + fmt::Display, V: ?Sized + fmt::Display>(
        &mut self,
        name: &K,
        value: &V,
    ) -> anyhow::Result<()> {
        self.buffer.clear();
        use fmt::Write;
        if write!(&mut self.buffer, "{name}").is_err() {
            bail!("convert query key to string");
        }
        let k_end = self.buffer.len();
        if write!(&mut self.buffer, "{value}").is_err() {
            bail!("convert query value to string");
        }
        let (name, value) = self.buffer.split_at(k_end);
        self.encoder.append_pair(name, value);
        Ok(())
    }

    pub fn display_pairs<I, K, V>(&mut self, query_pairs: I) -> anyhow::Result<()>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: fmt::Display,
        V: fmt::Display,
    {
        for item in query_pairs.into_iter() {
            let (name, value) = item.borrow();
            self.display_pair(name, value)?;
        }
        Ok(())
    }
}
