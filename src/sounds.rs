//------------------------------------------------
//------------WAVE DATA---------------------------
//----------------------square--------------------------
pub fn make_sine() -> Vec<f64> {
  let cycles_per_sample = super::FREQF / super::SAMPLE_RATE;
  let angle_delta = cycles_per_sample * std::f64::consts::PI * 2.0f64;
  (0..super::SAMPLE_RATE as usize)
      .map(|x| (angle_delta * x as f64) / super::SAMPLE_RATE)
      .map(|x| x.sin())
      .collect()
}

pub fn add_sine(signal : &mut Vec<f64>, freq: f64, amp: f64, phase: f64) {
  let twopi = std::f64::consts::PI * 2.0f64;

  // audiosignal[i]+= amp * sin((TWO_PI * (i*freq) / 512) + phase);
  for i in 0..super::SAMPLE_RATE as usize {
    let mut new_signal;
    new_signal = freq * i as f64;
    new_signal = new_signal / super::SAMPLE_RATE;
    new_signal = new_signal * twopi;
    new_signal += phase;
    new_signal = new_signal.sin() * amp; 
    signal[i] += new_signal;
  }
}

pub fn make_square() -> Vec<f64> {
  let wave = &mut make_sine();
  let updates = 3..(20_000.0f64 - super::FREQF) as i32;
  for i in updates.step_by(2) {
      let i_f = i as f64;
      add_sine(
          wave,
          super::FREQF + i_f,
          1.0 / i_f,
          0f64
      );
  }
  wave.to_vec()
}
