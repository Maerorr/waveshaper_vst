use nih_plug::prelude::*;
use waveshaper::ShapeType;
use std::{sync::{Arc, mpsc::channel}, collections::VecDeque, env, char::MAX};

use nih_plug::prelude::*;
use nih_plug_vizia::ViziaState;


mod lfo;
mod editor;
mod filter;
mod waveshaper;

const MAX_BLOCK_SIZE: usize = 64;


struct DistortionPlugin {
    params: Arc<DistortionPluginParams>,
    sample_rate: f32,
    waveshaper: waveshaper::Waveshaper,
    scr: Box<ScratchBuffers>,
}

struct ScratchBuffers {
    // These are for the Hard Vacuum parameters
    saturation: [f32; MAX_BLOCK_SIZE],
    pre_gain: [f32; MAX_BLOCK_SIZE],
    post_gain: [f32; MAX_BLOCK_SIZE],
}

impl Default for ScratchBuffers {
    fn default() -> Self {
        Self {
            saturation: [0.0; MAX_BLOCK_SIZE],
            pre_gain: [0.0; MAX_BLOCK_SIZE],
            post_gain: [0.0; MAX_BLOCK_SIZE],
        }
    }
}


#[derive(Params)]
struct DistortionPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<ViziaState>,

    #[id = "shape"]
    shape: EnumParam<ShapeType>,

    #[id = "pre-gain"]
    pre_gain: FloatParam,

    #[id = "post-gain"]
    post_gain: FloatParam,

    #[id = "saturation"]
    saturation: FloatParam,
}

impl Default for DistortionPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(DistortionPluginParams::default()),
            sample_rate: 44100.0,
            waveshaper: waveshaper::Waveshaper::new(ShapeType::SIG, 1.0, 1.0, 1.0),
            scr: Box::new(ScratchBuffers::default()),
        }
    }
}

impl Default for DistortionPluginParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),

            shape: EnumParam::new(
                "Shape",
                ShapeType::SIG,
            ),

            pre_gain: FloatParam::new(
                "pre-gain", 
                util::db_to_gain(0.0), 
                FloatRange::Skewed {
                min: util::db_to_gain(-30.0),
                max: util::db_to_gain(30.0),
                factor: FloatRange::gain_skew_factor(-30.0, 30.0),
            },)
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            post_gain: FloatParam::new(
                "post-gain", 
                util::db_to_gain(0.0), 
                FloatRange::Skewed {
                min: util::db_to_gain(-30.0),
                max: util::db_to_gain(0.0),
                factor: FloatRange::gain_skew_factor(-30.0, 0.0),
            },)
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            saturation: FloatParam::new(
                "saturation", 
                1.0, 
                FloatRange::Linear { min: 0.1, max: 10.0 }
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for DistortionPlugin {
    const NAME: &'static str = "tsk waveshaper";
    const VENDOR: &'static str = "236587 & 236598";
    const URL: &'static str = "none";
    const EMAIL: &'static str = "none";
    const VERSION: &'static str = "test";

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: NonZeroU32::new(2),
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = _buffer_config.sample_rate as f32;
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        
        // In current configuration this function iterates as follows:
        // 1. outer loop iterates block-size times
        // 2. inner loop iterates channel-size times. 

        for (_, block) in buffer.iter_blocks(MAX_BLOCK_SIZE) {
            let block_len = block.samples();

            let shape = self.params.shape.value();

            let saturation = &mut self.scr.saturation;
            self.params.saturation.smoothed.next_block(saturation, block_len);

            let pre_gain = &mut self.scr.pre_gain;
            self.params.pre_gain.smoothed.next_block(pre_gain, block_len);

            let post_gain = &mut self.scr.post_gain;
            self.params.post_gain.smoothed.next_block(post_gain, block_len);

            for block_channel in block.into_iter() {
                for (sample_idx, sample) in block_channel.into_iter().enumerate() {
                    self.waveshaper.set_params(
                        shape, 
                        saturation[sample_idx],
                        pre_gain[sample_idx],
                        post_gain[sample_idx],
                    );
                    *sample = self.waveshaper.process(*sample);
                }
            }
        }

        ProcessStatus::Normal
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(
            self.params.clone(),
            self.params.editor_state.clone(),
        )
    }
}

impl ClapPlugin for DistortionPlugin {
    const CLAP_ID: &'static str = "{{ cookiecutter.clap_id }}";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("{{ cookiecutter.description }}");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for DistortionPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"tsk__Wavshaper__";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Dynamics, Vst3SubCategory::Distortion];
}

//nih_export_clap!(MaerorChorus);
nih_export_vst3!(DistortionPlugin);
