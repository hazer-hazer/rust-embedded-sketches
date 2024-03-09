use super::{AudioSource, AudioSourceProps, Sample};

pub fn mix<I1, I2>(input1: I1, input2: I2) -> Mix<I1, I2>
where
    I1: AudioSource,
    I1::Item: Sample,
    I2: AudioSource,
    I2::Item: Sample,
{
    return Mix::new(input1, input2);
}

pub struct Mix<I1, I2>
where
    I1: AudioSource,
    I1::Item: Sample,
    I2: AudioSource,
    I2::Item: Sample,
{
    input1: I1,
    input2: I2,
    props: AudioSourceProps,
}

impl<I1, I2> Mix<I1, I2>
where
    I1: AudioSource,
    I1::Item: Sample,
    I2: AudioSource,
    I2::Item: Sample,
{
    pub fn new(input1: I1, input2: I2) -> Self {
        assert!(input1.sample_rate() == input2.sample_rate());

        // TODO: Sample type conversion + Channel unification + resampling

        let props = AudioSourceProps::new(
            input1.channels(),
            input1.sample_rate(),
            input1.total_duration(),
        );

        Self {
            input1,
            input2,
            props,
        }
    }
}

impl<I1, I2> AudioSource for Mix<I1, I2>
where
    I1: AudioSource,
    I1::Item: Sample + From<I2::Item>,
    I2: AudioSource,
    I2::Item: Sample,
{
    fn props(&self) -> super::AudioSourceProps {
        self.props
    }
}

impl<I1, I2> Iterator for Mix<I1, I2>
where
    I1: AudioSource,
    I1::Item: Sample + From<I2::Item>,
    I2: AudioSource,
    I2::Item: Sample,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.input1.next(), self.input2.next()) {
            (None, None) => None,
            (None, Some(only2)) => Some(only2.into()),
            (Some(only1), None) => Some(only1),
            (Some(input1), Some(input2)) => Some(input1.saturating_add(input2.into())),
        }
    }
}
