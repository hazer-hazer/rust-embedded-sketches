use crate::dsp::sample::Sample;

use super::AudioSource;

pub struct Restarting<S: AudioSource>
where
    S::Item: Sample,
{
    restart: bool,
    pub inner: S,
}

impl<S: AudioSource> Restarting<S>
where
    S::Item: Sample,
{
    pub fn new(inner: S) -> Self {
        Self {
            restart: true,
            inner,
        }
    }

    pub fn set_restarting(&mut self, restart: bool) {
        self.restart = restart;
    }

    pub fn is_restarting(&self) -> bool {
        self.restart
    }
}

impl<S: AudioSource> Iterator for Restarting<S>
where
    S::Item: Sample,
{
    type Item = S::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().or_else(|| {
            if self.restart {
                self.inner.restart();
                Some(
                    self.inner
                        .next()
                        .expect("AudioSource MUST give samples after restart"),
                )
            } else {
                None
            }
        })
    }
}

impl<S: AudioSource> AudioSource for Restarting<S>
where
    S::Item: Sample,
{
    #[inline]
    fn props(&self) -> super::AudioSourceProps {
        self.inner.props()
    }

    #[inline]
    fn props_mut(&mut self) -> &mut super::AudioSourceProps {
        self.inner.props_mut()
    }

    #[inline]
    fn is_finished(&self) -> bool {
        !self.restart || self.inner.is_finished()
    }
}
