use std::sync::Arc;

use nih_plug::prelude::{util, Editor, Vst3Plugin, EnumParam};
use nih_plug_vizia::vizia::image::Pixel;
use nih_plug_vizia::vizia::prelude::*;
use nih_plug_vizia::widgets::*;
use nih_plug_vizia::{assets, create_vizia_editor, ViziaState, ViziaTheming};


use crate::DistortionPluginParams;


#[derive(Lens)]
struct Data {
    distortion_data: Arc<DistortionPluginParams>
}

impl Model for Data {}

pub(crate) fn default_state() -> Arc<ViziaState> {
    ViziaState::new(|| (400, 300))
}

pub(crate) fn create(
    distortion_data: Arc<DistortionPluginParams>,
    editor_state: Arc<ViziaState>,
) -> Option<Box<dyn Editor>> {
    create_vizia_editor(editor_state, 
        ViziaTheming::Custom, move |cx, _| {
            assets::register_noto_sans_light(cx);
            assets::register_noto_sans_thin(cx);

            Data {
                distortion_data: distortion_data.clone(),
            }.build(cx);

            ResizeHandle::new(cx);

            VStack::new(cx, |cx: &mut Context| {
                Label::new(cx, "FLANGER/VIBRATO")
                .font_family(vec![FamilyOwned::Name(String::from(
                    assets::NOTO_SANS_THIN,
                ))])
                .font_size(30.0)
                .height(Pixels(50.0))
                .child_top(Stretch(1.0))
                .child_bottom(Pixels(30.0));
                
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(cx, "shape")
                        .font_size(15.0)
                        .height(Pixels(30.0));

                        Label::new(cx, "saturation")
                        .font_size(15.0)
                        .height(Pixels(30.0));

                        Label::new(cx, "pre-gain")
                        .font_size(15.0)
                        .height(Pixels(30.0));

                        Label::new(cx, "post-gain")
                        .font_size(15.0)
                        .height(Pixels(30.0));                        

                    }).child_top(Pixels(6.0))
                    .row_between(Pixels(3.0));
    
                    VStack::new(cx, |cx| {
                        ParamSlider::new(cx, Data::distortion_data, |params| &params.shape)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::distortion_data, |params| &params.saturation)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::distortion_data, |params| &params.pre_gain)
                        .height(Pixels(30.0));

                        ParamSlider::new(cx, Data::distortion_data, |params| &params.post_gain)
                        .height(Pixels(30.0));

                    }).row_between(Pixels(3.0));

                }).col_between(Pixels(30.0));
                
            }).row_between(Pixels(0.0))
            .child_left(Stretch(1.0))
            .child_right(Stretch(1.0));

        })
}