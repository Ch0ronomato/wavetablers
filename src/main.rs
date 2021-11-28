extern crate coreaudio;

use std::sync::{Arc, Mutex};
use std::convert::TryInto;
use coreaudio::audio_unit::{AudioUnit, IOType, SampleFormat};
use coreaudio::audio_unit::render_callback::{self, data};
use structopt::StructOpt;
use util::apple_said_yes;
use util::apple_said_no;

mod console;
mod sounds;
mod util;

static mut FINALIZED_DATA:Vec<f64> = vec![];
static mut SHOULD_MUTE:f32 = 0.0f32;
const SAMPLE_RATE: f64 = 44_100.0f64;

struct Wavetable {
    frequency: std::vec::Vec<f32>,
    frequency_phase: std::vec::Vec<f32>,
    audio_data_index: f32,
    table_size: usize,
    shape: f32,
}

impl Wavetable {
    pub fn new(should_mute: bool, will_plot: bool) -> Wavetable {
        /**
         * An experiment:
         *
         * Sample rate is measured currently as 44_100hz, which means we need
         * to provide the a measure of the wavetable's value for one second in
         * 44_100 pieces in order to render the sound.
         *
         * The sound of A is a 440hz, which is just a wave that repeating 440 a second.
         *
         * If we treat the type of wave as a square wave, which a 50% duty cycle (that means
         * that is 1 for 50% of the period), then we should, in theory be able to "embed" a square
         * wave right into the wavetable without having to read a .aiff or .wav file from disk.
         *
         * Why? We want to cement the understanding that we need some way of taking the note value
         * and translating it into the sample rate amount, and that doesn't mean that the sound has
         * magically moved up to that sample rate. You can see that this confusion exists currently
         * in the sounds module, where `make_sine` is essentially 10000 pieces of a sine wave, and
         * then we do an awkward jump during the callback leading to artifacts.
         */
        Wavetable {
            frequency: vec![0f32; 1],
            frequency_phase: vec![0f32; 1],
            table_size: 2i32.pow(12) as usize,
            audio_data_index: 0.0f32,
            shape: 0.5f32,
        }
    }

    // the formula listed in the iterator is useful to understand how this new stack now works
    // but essentially, we're just changing the frequency amount of this particular function
    pub fn stack(&mut self, new_frequency: f32) -> Option<bool> {
        // not sure how to combine two separate notes. I think it's addition
        println!("{}", apple_said_yes("Stacking"));        
        self.frequency.push(new_frequency);
        // voice amount will need to be defined
        Some(self.frequency.len() < 5)
    }

    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency.clear();
        self.frequency_phase.clear();
        self.frequency.push(frequency);
        let fundamental = self.table_size as f32 / SAMPLE_RATE as f32;
        self.frequency_phase.push(frequency * fundamental);
    }
}

impl Iterator for Wavetable {
    type Item = f32;
    // this iterator does *not* run for `table_size` times, it actually runs
    // for the sample_rate, but returns cyclic to the wave table
    //
    // If `frequency` is 440, and our sample rate is 44_100, and our table size is 128,
    // then the number of times we cycle the wavetable per second equals
    //
    // `frequency` * (`table_size` / `sample_rate`) =
    //   440       * (   128       /   44_100     ) =
    //            1.277097506 
    fn next(&mut self) -> Option<f32> {
        let mut current_sample = 0f32;
        let table_size_f32 = self.table_size as i32 as f32;
        for (_, phase) in self.frequency.iter().zip(self.frequency_phase.iter()) {
            let new_audio_data_index = self.audio_data_index + phase;
            current_sample = if new_audio_data_index < table_size_f32 * self.shape { -1.0f32 } else { 1.0f32 };
            self.audio_data_index = new_audio_data_index;
            if self.audio_data_index > table_size_f32 {
                self.audio_data_index -= table_size_f32;
            }
            self.shape -= 0.0001f32;
            if self.shape < 0.05f32 {
                self.shape = 0.5f32;
            }
        }
        Some(current_sample)
    }
}

fn main() -> Result<(), coreaudio::Error>{
    let args = util::Cli::from_args();
    const A: f32 = 440.0f32;
    const Db: f32 = 554.37f32;
    const E: f32 = 659.25f32;

    let mut samples = Wavetable::new(!args.nomute, args.plot);
    samples.set_frequency(A);
    // samples.stack(Db / 2.0f64);
    // samples.stack(E / 2.0f32);
    //
    // Construct an Output audio unit that delivers audio to the default output device.
    let mut audio_unit = AudioUnit::new(IOType::DefaultOutput)?;
    audio_unit.set_sample_rate(SAMPLE_RATE);

    let stream_format = audio_unit.output_stream_format()?;
    println!("{}", apple_said_yes(&format!("{:#?}", &stream_format)));

    assert!(SampleFormat::F32 == stream_format.sample_format);

    type Args = render_callback::Args<data::NonInterleaved<f32>>;
    let export = Arc::new(Mutex::new(std::vec::Vec::<f32>::new()));
    let writer = export.clone();
    // Inspiration for the redesign
    // This is a separate thread.
    //
    // let mut u = format!("{:?}", std::thread::current().id());
    // u = apple_said_no(&u);
    // println!("{}", u);
    //
    // if you execute this code in both the render callback and outside
    // you'll get two separate thread IDs. That means that whatever we 
    // implement, we actually need to have it be able to cross thread
    // boundaries.
    //
    // Interestingly, we need to have an exclusive reference to use an 
    // iterator in Rust. I think that maybe the core of the issue here. 
    // We need to have some type that is safe to use across thread boundaries
    // but in addition to that, we need to also have it only need a immutable reference
    audio_unit.set_render_callback(move |args| {
        let Args { num_frames, mut data, .. } = args;
        for i in 0..num_frames {
            let sample = samples.next().unwrap();
            if let Ok(mut i) = writer.lock() {
                i.push(sample);
            }
            for channel in data.channels_mut() {
                channel[i] = sample;
            }
        }
        Ok(())
    })?;
    let res = audio_unit.start();

    std::thread::sleep(std::time::Duration::from_millis(10000));
    audio_unit.stop()?;
    if let Ok(i) = export.lock() {
        println!("{:?}", i);
    }
    /*
    println!("{}", apple_said_yes(&"Adding Db"));
    std::thread::sleep(std::time::Duration::from_millis(3000));
    println!("{}", apple_said_yes(&"Adding E"));
    std::thread::sleep(std::time::Duration::from_millis(3000));
    */

    res
}
