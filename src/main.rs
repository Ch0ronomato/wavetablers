extern crate coreaudio;

use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coreaudio::audio_unit::render_callback::{self, data};
use structopt::StructOpt;
use std::fs;
use util::apple_said_yes;
use util::apple_said_no;

mod console;
mod sounds;
mod util;

static mut FINALIZED_DATA:Vec<f64> = vec![];
static mut SHOULD_MUTE:f32 = 0.0f32;

struct Wavetable {
    audio_data: Vec<f32>,
    audio_data_index: usize,
    should_mute: bool,
    will_plot: bool
}

impl Wavetable {
    pub fn new(audio_data: Vec<f32>, should_mute: bool, will_plot: bool) -> Wavetable {
        Wavetable {
            audio_data,
            audio_data_index: 0,
            should_mute,
            will_plot
        }
    }
}

impl Iterator for Wavetable {
    type Item = f32;
    fn next(&mut self) -> Option<f32> {
        let next = self.audio_data[self.audio_data_index] * (if self.should_mute { 0.0f32 } else { 0.05f32 });
        if self.will_plot {
            unsafe {
                FINALIZED_DATA.push(self.audio_data[self.audio_data_index] as f64)
            }
        }
        self.audio_data_index = (self.audio_data_index + 1) % self.audio_data.len();
        Some(next)
    }
}

// Note: 440 is A
const FREQF: f64 = 440.0f64;
const SAMPLE_RATE: f64 = 48_000.0f64;

fn main() -> Result<(), coreaudio::Error>{
    let args = util::Cli::from_args();
    let mut samples = Wavetable::new(sounds::make_sine().iter().map(|x| *x as f32).collect(), !args.nomute, args.plot);
    //
    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;

    let stream_format = audio_unit.output_stream_format()?;
    println!("{}", apple_said_yes(&format!("{:#?}", &stream_format)));

    // For this example, our sine wave expects `f32` data.
    assert!(SampleFormat::F32 == stream_format.sample_format);

    type Args = render_callback::Args<data::NonInterleaved<f32>>;
    audio_unit.set_render_callback(move |args| {
        let Args { num_frames, mut data, .. } = args;
        for i in 0..num_frames {
            let sample = samples.next().unwrap();
            for channel in data.channels_mut() {
                channel[i] = sample;
            }
        }
        Ok(())
    })?;
    let res = audio_unit.start();

    std::thread::sleep(std::time::Duration::from_millis(3000));

    if args.plot {
        // Print.
        println!("{}", apple_said_yes("Plotting"));
        unsafe {
            console::draw_console(&FINALIZED_DATA);
        }
    } else {
        println!("{}", apple_said_no("Skipping plotting"));
    }

    res
}
