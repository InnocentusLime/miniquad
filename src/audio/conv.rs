use std::iter::FusedIterator;

use glam::{I16Vec2, i16vec2};

use crate::audio::{Error, Result};

/// Converts audio sample data into the same uniform
/// internal format.
pub fn normalize_audio<I>(samples: I, spec: hound::WavSpec) -> Result<Vec<I16Vec2>>
where
    I: Iterator<Item = hound::Result<i16>>,
{
    let scaled = samples
        .map(|x| x.map_err(Error::WavParsingError))
        .map(|x| scale_samples(x, spec));
    ChannelIter { spec, iter: scaled }.collect()
}

fn scale_samples(x: Result<i16>, spec: hound::WavSpec) -> Result<i16> {
    let x = x?;
    match spec.bits_per_sample {
        8 => Ok(x.saturating_mul(i8::MAX as i16)),
        16 => Ok(x),
        found => Err(Error::UnexpectedBitdepth { found }),
    }
}

struct ChannelIter<I> {
    spec: hound::WavSpec,
    iter: I,
}

impl<I: FusedIterator<Item = Result<i16>>> FusedIterator for ChannelIter<I> {}

impl<I: Iterator<Item = Result<i16>>> Iterator for ChannelIter<I> {
    type Item = Result<I16Vec2>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.spec.channels {
            1 => next_mono(&mut self.iter),
            2 => next_stereo(&mut self.iter),
            found => Some(Err(Error::UnexpectedChannelCount { found })),
        }
    }
}

fn next_mono<I>(iter: &mut I) -> Option<Result<I16Vec2>>
where
    I: Iterator<Item = Result<i16>>,
{
    iter.next().map(|x| x.map(I16Vec2::splat))
}

fn next_stereo<I>(iter: &mut I) -> Option<Result<I16Vec2>>
where
    I: Iterator<Item = Result<i16>>,
{
    let l: i16;
    match iter.next() {
        None => return None,
        Some(Err(e)) => return Some(Err(e)),
        Some(Ok(x)) => l = x,
    }

    let r: i16;
    match iter.next() {
        None => return Some(Err(Error::UnexpectedWAVEOF)),
        Some(Err(e)) => return Some(Err(e)),
        Some(Ok(x)) => r = x,
    }

    Some(Ok(i16vec2(l, r)))
}
