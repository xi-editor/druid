// Copyright 2019 The xi-editor Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! An example of an animating widget.

use std::f64::consts::PI;

use druid::kurbo::{Affine, Circle, Line};
use druid::widget::{Align, Button, Checkbox, Flex, Label, Padding, ProgressBar, Slider};
use druid::{
    AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, PaintCtx, Point,
    RenderContext, Size, UpdateCtx, Vec2, Widget, WindowDesc,
};

struct AnimWidget {
    t: f64,
}

impl Widget<u32> for AnimWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut u32, _env: &Env) {
        match event {
            Event::MouseDown(_) => {
                self.t = 0.0;
                ctx.request_anim_frame();
            }
            Event::AnimFrame(interval) => {
                self.t += (*interval as f64) * 1e-9;
                if self.t < 1.0 {
                    ctx.request_anim_frame();
                }
                // When we do fine-grained invalidation,
                // no doubt this will be required:
                //ctx.invalidate();
            }
            _ => (),
        }
    }

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&u32>, _data: &u32, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &u32,
        _env: &Env,
    ) -> Size {
        bc.constrain((100.0, 100.0))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &u32, _env: &Env) {
        let center = Point::new(50.0, 50.0);
        let ambit = center + 45.0 * Vec2::from_angle((0.75 + self.t) * 2.0 * PI);
        paint_ctx.stroke(Line::new(center, ambit), &Color::WHITE, 1.0);

        paint_ctx.fill(Circle::new(Point::new(100.0, 100.0), 20.0), &Color::WHITE);

        paint_ctx.paint_with_z_index(3, |ctx| {
            ctx.fill(
                Circle::new(Point::new(140.0, 100.0), 20.0),
                &Color::rgb8(255, 255, 0),
            );
        });

        paint_ctx.paint_with_z_index(1, |ctx| {
            ctx.fill(
                Circle::new(Point::new(120.0, 100.0), 20.0),
                &Color::rgb8(255, 0, 0),
            );
        });

        paint_ctx.fill(
            Circle::new(Point::new(100.0, 120.0), 20.0),
            &Color::rgb8(255, 0, 255),
        );
    }
}

struct CircleWidget {}

impl Widget<u32> for CircleWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, _data: &mut u32, _env: &Env) {}
    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: Option<&u32>, _data: &u32, _env: &Env) {}

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &u32,
        _env: &Env,
    ) -> Size {
        bc.constrain((500.0, 500.0))
    }

    fn paint(&mut self, paint_ctx: &mut PaintCtx, _data: &u32, _env: &Env) {
        paint_ctx.transform(Affine::translate((-50., 0.)));

        paint_ctx.fill(
            Circle::new(Point::new(125.0, 70.0), 20.0),
            &Color::rgb8(125, 50, 55),
        );

        paint_ctx.paint_with_z_index(2, |ctx| {
            ctx.fill(
                Circle::new(Point::new(135.0, 70.0), 20.0),
                &Color::rgb8(0, 0, 255),
            );
        });
    }
}

fn build_widget() -> impl Widget<u32> {
    let anim_widget = AnimWidget { t: 0.0 };
    let circle_widget = CircleWidget {};

    Flex::column()
        .with_child(Padding::new(0.0, anim_widget), 1.0)
        .with_child(Padding::new(0.0, circle_widget), 1.0)
}

fn main() {
    let window = WindowDesc::new(build_widget);
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(0)
        .expect("launch failed");
}
