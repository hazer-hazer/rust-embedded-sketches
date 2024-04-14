use crate::dsp::filter::{all_pass::AllPass, comb::CombFilter};

use super::effect::Effect;

const FIXED_GAIN: f64 = 0.015;

const SCALE_WET: f64 = 3.0;
const SCALE_DAMPENING: f64 = 0.4;

const SCALE_ROOM: f64 = 0.28;
const OFFSET_ROOM: f64 = 0.7;

const STEREO_SPREAD: usize = 23;

const COMB_TUNING_L1: usize = 1116;
const COMB_TUNING_R1: usize = 1116 + STEREO_SPREAD;
const COMB_TUNING_L2: usize = 1188;
const COMB_TUNING_R2: usize = 1188 + STEREO_SPREAD;
const COMB_TUNING_L3: usize = 1277;
const COMB_TUNING_R3: usize = 1277 + STEREO_SPREAD;
const COMB_TUNING_L4: usize = 1356;
const COMB_TUNING_R4: usize = 1356 + STEREO_SPREAD;
const COMB_TUNING_L5: usize = 1422;
const COMB_TUNING_R5: usize = 1422 + STEREO_SPREAD;
const COMB_TUNING_L6: usize = 1491;
const COMB_TUNING_R6: usize = 1491 + STEREO_SPREAD;
const COMB_TUNING_L7: usize = 1557;
const COMB_TUNING_R7: usize = 1557 + STEREO_SPREAD;
const COMB_TUNING_L8: usize = 1617;
const COMB_TUNING_R8: usize = 1617 + STEREO_SPREAD;

const ALLPASS_TUNING_L1: usize = 556;
const ALLPASS_TUNING_R1: usize = 556 + STEREO_SPREAD;
const ALLPASS_TUNING_L2: usize = 441;
const ALLPASS_TUNING_R2: usize = 441 + STEREO_SPREAD;
const ALLPASS_TUNING_L3: usize = 341;
const ALLPASS_TUNING_R3: usize = 341 + STEREO_SPREAD;
const ALLPASS_TUNING_L4: usize = 225;
const ALLPASS_TUNING_R4: usize = 225 + STEREO_SPREAD;

pub struct FreeVerb {
    combs: [(CombFilter, CombFilter); 8],
    all_passes: [(AllPass, AllPass); 4],
    wet_gains: (f32, f32),
    wet: f32,
    width: f32,
    dry: f32,
    input_gain: f32,
    dampening: f32,
    room_size: f32,
    frozen: bool,
}

// TODO

impl FreeVerb {
    pub fn new(sr: usize) -> Self {
        let adjust_len = |len: usize| (len as f32 * sr as f32 / 44100.0) as usize;

        FreeVerb {
            combs: [
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L1)),
                    CombFilter::new(adjust_len(COMB_TUNING_R1)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L2)),
                    CombFilter::new(adjust_len(COMB_TUNING_R2)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L3)),
                    CombFilter::new(adjust_len(COMB_TUNING_R3)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L4)),
                    CombFilter::new(adjust_len(COMB_TUNING_R4)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L5)),
                    CombFilter::new(adjust_len(COMB_TUNING_R5)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L6)),
                    CombFilter::new(adjust_len(COMB_TUNING_R6)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L7)),
                    CombFilter::new(adjust_len(COMB_TUNING_R7)),
                ),
                (
                    CombFilter::new(adjust_len(COMB_TUNING_L8)),
                    CombFilter::new(adjust_len(COMB_TUNING_R8)),
                ),
            ],
            all_passes: [
                (
                    AllPass::new(adjust_len(ALLPASS_TUNING_L1)),
                    AllPass::new(adjust_len(ALLPASS_TUNING_R1)),
                ),
                (
                    AllPass::new(adjust_len(ALLPASS_TUNING_L2)),
                    AllPass::new(adjust_len(ALLPASS_TUNING_R2)),
                ),
                (
                    AllPass::new(adjust_len(ALLPASS_TUNING_L3)),
                    AllPass::new(adjust_len(ALLPASS_TUNING_R3)),
                ),
                (
                    AllPass::new(adjust_len(ALLPASS_TUNING_L4)),
                    AllPass::new(adjust_len(ALLPASS_TUNING_R4)),
                ),
            ],
            wet_gains: (0.0, 0.0),
            wet: 1.0,
            dry: 0.0,
            input_gain: 0.0,
            width: 0.5,
            dampening: 0.5,
            room_size: 0.5,
            frozen: false,
        }
    }
}

// impl Stereo for FreeVerb {
//     fn process(&mut self, input: f32) -> f32 {

//     }
// }
