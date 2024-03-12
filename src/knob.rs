use crate::*;

/// Type alias for an SAADC with static lifetime and 1 channel.
pub type Adc = saadc::Saadc<'static, 1>;

/// A Knob that is a wrapper for an Adc
pub struct Knob(Adc);

impl Knob {
    /// Creates a new knob with the provided adc.
    ///
    /// # Arguments
    ///
    /// * 'adc' - An ADC to be used to read measurements.
    ///
    /// # Returns
    ///
    /// A new Knob instance.
    pub async fn new(adc: Adc) -> Self {
        adc.calibrate().await; // calibrate the provided adc
        Self(adc) // return the new instance
    }

    /// Takes a measurement of the current adc status,
    /// scales it and returns it as a u32.
    ///
    /// # Returns
    ///
    /// A u32 value as a scaled value.
    pub async fn measure(&mut self) -> u32 {
        // Take a sample from the single channel and store it in a buffer.
        // The sample will be an i16 value.
        let mut buf = [0];
        self.0.sample(&mut buf).await;

        // Clamp the value to a max and convert it to a u16 from i16.
        let raw = buf[0].clamp(0, 0x7fff) as u16;

        // Scale the raw value by converting to f32 and normalizing
        // to between 0 and 1.
        let scaled = raw as f32 / 10_000.0;

        // Rescale to the range specified by LEVELS.
        let result = ((LEVELS + 2) as f32 * scaled - 2.0)
            .clamp(0.0, (LEVELS - 1) as f32)
            .floor();

        result as u32
    }
}
