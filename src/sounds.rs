//------------------------------------------------
//------------WAVE DATA---------------------------
//----------------------square--------------------------
pub fn make_sine(freqf: f64) -> Vec<f64> {
  let time_delta = 1.0f64 / super::SAMPLE_RATE;
  let angle_delta = time_delta * std::f64::consts::PI * 2.0f64;
  // essentially we're having fixed time here. In the future, this will
  // probably need to be a slightly continious value (think of a tick)
  (0..10000 as usize)
      .map(|x| (freqf * angle_delta * x as f64))
      .map(|x| x.sin())
      .collect()
}

pub fn add_sine(signal : &mut Vec<f64>, freq: f64, amp: f64, phase: f64) {
  let twopi = std::f64::consts::PI * 2.0f64;
  let time_delta = 1.0f64 / super::SAMPLE_RATE;
  let angle_delta = time_delta * twopi;

  // audiosignal[i]+= amp * sin((TWO_PI * (i*freq) / 512) + phase);
  for i in 0..10000 {
    let mut new_signal;
    new_signal = angle_delta * freq * (i as f64);
    new_signal += phase;
    new_signal = new_signal.sin() * amp; 
    signal[i] += new_signal;
  }
}

pub fn make_square(freqf: f64) -> Vec<f64> {
  let wave = &mut make_sine(freqf);
  // let updates = 3..(20_000.0f64 - super::FREQF) as i32;
  let updates = 3..14;
  for i in updates.step_by(2) {
      let i_f = i as f64;
      add_sine(
          wave,
          freqf + i_f,
          1.0 / i_f,
          0f64
      );
  }
  wave.to_vec()
}
