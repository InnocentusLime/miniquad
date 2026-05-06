use std::iter::FusedIterator;

use glam::{I16Vec2, i16vec2};

use crate::audio::{Error, Result};

/// Converts audio sample data into the same uniform
/// internal format.
pub fn normalize_audio<I>(samples: I, spec: hound::WavSpec) -> Result<Vec<I16Vec2>>
where
    I: Iterator<Item = hound::Result<i16>>,
{
    let sample_scaler: fn(i16) -> i16 = match spec.bits_per_sample {
        8 => |x| x.saturating_mul(i8::MAX as i16),
        16 => |x| x,
        found => return Err(Error::UnexpectedBitdepth { found }),
    };

    let scaled = samples
        .map(|x| x.map_err(Error::WavParsingError))
        .map(|x| x.map(sample_scaler));

    match spec.channels {
        1 => scaled.map(|x| x.map(I16Vec2::splat)).collect(),
        2 => ChannelIter { iter: scaled }.collect(),
        found => Err(Error::UnexpectedChannelCount { found }),
    }
}

struct ChannelIter<I> {
    iter: I,
}

impl<I: FusedIterator<Item = Result<i16>>> FusedIterator for ChannelIter<I> {}

impl<I: Iterator<Item = Result<i16>>> Iterator for ChannelIter<I> {
    type Item = Result<I16Vec2>;

    fn next(&mut self) -> Option<Self::Item> {
        let l: i16;
        match self.iter.next() {
            None => return None,
            Some(Err(e)) => return Some(Err(e)),
            Some(Ok(x)) => l = x,
        }

        let r: i16;
        match self.iter.next() {
            None => return Some(Err(Error::UnexpectedWAVEOF)),
            Some(Err(e)) => return Some(Err(e)),
            Some(Ok(x)) => r = x,
        }

        Some(Ok(i16vec2(l, r)))
    }
}
