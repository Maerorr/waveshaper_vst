use std::f32::consts::E;

use nih_plug::prelude::Enum;


#[derive(Clone, Copy, PartialEq)]
pub enum ShapeType {
    ARRY,
    SIG, // USES SATURATION
    SIG2,
    TANH,// USES SATURATION
    ATAN,// USES SATURATION
    FEXP1,// USES SATURATION
    FEXP2,
    EXP,
    ATSR,
    SQS,
    CUBE,
    HCLIP,
    HWR,
    FWR,
    ASQRT,
}

impl Enum for ShapeType {
    fn variants() -> &'static [&'static str] {
        &[
            "arraya",
            "sigmoid",
            "sigmoid 2",
            "hyperbolic tangent",
            "arctangent",
            "fuzz exponential 1",
            "fuzz exponential 2",
            "exponential",
            "atan square root",
            "square sign",
            "cube",
            "hardclip",
            "half wave rectifier",
            "full wave rectifier",
            "absolute square root",
        ]
    }

    fn ids() -> Option<&'static [&'static str]> {
        Some(&[
            "arry",
            "sig",
            "sig2",
            "tanh",
            "atan",
            "fexp1",
            "fexp2",
            "exp",
            "atsr",
            "sqs",
            "cube",
            "hclip",
            "hwr",
            "fwr",
            "asqrt",
        ])
    }

    fn to_index(self) -> usize {
        match self {
            ShapeType::ARRY => 0,
            ShapeType::SIG => 1,
            ShapeType::SIG2 => 2,
            ShapeType::TANH => 3,
            ShapeType::ATAN => 4,
            ShapeType::FEXP1 => 5,
            ShapeType::FEXP2 => 6,
            ShapeType::EXP => 7,
            ShapeType::ATSR => 8,
            ShapeType::SQS => 9,
            ShapeType::CUBE => 10,
            ShapeType::HCLIP => 11,
            ShapeType::HWR => 12,
            ShapeType::FWR => 13,
            ShapeType::ASQRT => 14,
        }
    }

    fn from_index(index: usize) -> Self {
        match index {
            0 => ShapeType::ARRY,
            1 => ShapeType::SIG,
            2 => ShapeType::SIG2,
            3 => ShapeType::TANH,
            4 => ShapeType::ATAN,
            5 => ShapeType::FEXP1,
            6 => ShapeType::FEXP2,
            7 => ShapeType::EXP,
            8 => ShapeType::ATSR,
            9 => ShapeType::SQS,
            10 => ShapeType::CUBE,
            11 => ShapeType::HCLIP,
            12 => ShapeType::HWR,
            13 => ShapeType::FWR,
            14 => ShapeType::ASQRT,
            _ => panic!("Invalid index for ShapeType"),
        }
    }
}

// implement all of these as functions
pub fn sgn(x: f32) -> f32 {
    if x >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

pub fn arry(x: f32) -> f32 {
    (3.0 * x / 2.0) 
    * (1.0 - x*x/3.0)
}

pub fn sig(x: f32, saturation: f32) -> f32 {
    2.0 / (1.0 + (-saturation * x).exp()) - 1.0
}

pub fn sig2(x: f32) -> f32 {
    ((x.exp() - 1.0) * (E + 1.0))
    / 
    ((x.exp() + 1.0) * (E - 1.0))
}

pub fn tanh(x: f32, saturation: f32) -> f32 {
    (saturation * x).tanh() / saturation.tanh()
}

pub fn atan(x: f32, saturation: f32) -> f32 {
    (saturation * x).atan() / (saturation).atan()
}

pub fn fexp1(x: f32, saturation: f32) -> f32 {
    sgn(x) * (
        (1.0 - (-(saturation * x).abs()).exp())
        /
        (1.0 - (-saturation).exp())
    )
}

pub fn fexp2(x: f32) -> f32 {
    sgn(x) * (
        (1.0 - (x.abs()).exp())
        /
        (E - 1.0)
    )
}

pub fn exp(x: f32) -> f32 {
    (E - (1.0 - x).exp()) / (E - 1.0)
}

pub fn atsr(x: f32) -> f32 {
    2.5 * (0.9*x).atan() + 2.5 * (1.0 - (0.9*x)*(0.9*x)).sqrt() -2.5
}

pub fn sqs(x: f32) -> f32 {
    x*x*sgn(x)
}

pub fn cube(x: f32) -> f32 {
    x*x*x
}

pub fn hclip(x: f32) -> f32 {
    if x.abs() > 0.5 {
        0.5 * sgn(x)
    } else {
        x
    }
}

pub fn hwr(x: f32) -> f32 {
    0.5*(x + x.abs())
}

pub fn fwr(x: f32) -> f32 {
    x.abs()
}

pub fn asqrt(x: f32) -> f32 {
    sgn(x) * x.abs().sqrt()
}

#[derive(Clone)]
pub struct Waveshaper {
    shape_type: ShapeType,
    saturation: f32,
    pre_gain: f32,
    post_gain: f32,
}

impl Waveshaper {
    pub fn new(shape_type: ShapeType, saturation: f32, pre_gain: f32, post_gain: f32) -> Self {
        Self {
            shape_type,
            saturation,
            pre_gain,
            post_gain,
        }
    }

    pub fn set_params(&mut self, shape_type: ShapeType, saturation: f32, pre_gain: f32, post_gain: f32) {
        self.shape_type = shape_type;
        self.saturation = saturation;
        self.pre_gain = pre_gain;
        self.post_gain = post_gain;
    }

    pub fn process(&self, input: f32) -> f32 {
        // convert self.pre_gain from decibels to line
        // let pre_gain = 10.0_f32.powf(self.pre_gain / 20.0);
        // let post_gain = 10.0_f32.powf(self.post_gain / 20.0);
        let x = input * self.pre_gain;
        let y = match self.shape_type {
            ShapeType::ARRY => arry(x),
            ShapeType::SIG => sig(x, self.saturation),
            ShapeType::SIG2 => sig2(x),
            ShapeType::TANH => tanh(x, self.saturation),
            ShapeType::ATAN => atan(x, self.saturation),
            ShapeType::FEXP1 => fexp1(x, self.saturation),
            ShapeType::FEXP2 => fexp2(x),
            ShapeType::EXP => exp(x),
            ShapeType::ATSR => atsr(x),
            ShapeType::SQS => sqs(x),
            ShapeType::CUBE => cube(x),
            ShapeType::HCLIP => hclip(x),
            ShapeType::HWR => hwr(x),
            ShapeType::FWR => fwr(x),
            ShapeType::ASQRT => asqrt(x),
        };
        y * self.post_gain
    }
}