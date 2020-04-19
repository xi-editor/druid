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

//! An example of a blocking function running in another thread.


use std::{thread, time};

use druid::{
	AppLauncher, Widget, WindowDesc, Selector, AppDelegate,
	ExtEventSink, Command, Data, Env, Lens, LocalizedString,
	DelegateCtx, Target, WidgetExt
};

use druid::widget::{Button, Either, Flex, Label};

const START_SLOW_FUNCTION: Selector = Selector::new("start_slow_function");

const FINISH_SLOW_FUNCTION: Selector = Selector::new("finish_slow_function");

struct Delegate {
    eventsink: ExtEventSink,
}

#[derive(Clone, Default, Data, Lens)]
struct AppState {
    processing: bool,
    value: u32,
}



// Pretend this is downloading a file, or doing heavy calculations...
fn slow_function(number: u32) -> u32 {
    let a_while = time::Duration::from_millis(2000);
    thread::sleep(a_while);
    number + 1
}

fn wrapped_slow_function(sink: ExtEventSink, number: u32) {
    thread::spawn(move || {
        let number = slow_function(number);
        sink.submit_command(FINISH_SLOW_FUNCTION, number, None)
            .expect("command failed to submit");
    });
}

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut DelegateCtx,
        _target: &Target,
        cmd: &Command,
        data: &mut AppState,
        _env: &Env,
    ) -> bool {
        match cmd.selector {
            START_SLOW_FUNCTION => {
                data.processing = true;
                wrapped_slow_function(self.eventsink.clone(), data.value);
                true
            }
            FINISH_SLOW_FUNCTION => {
                data.processing = false;
                let number = cmd.get_object::<u32>().expect("api violation");
                data.value = *number;
                true
            }
            _ => true,
        }
    }
}

fn ui_builder() -> impl Widget<AppState> {
    let button = Button::new("Start slow increment")
        .on_click(|ctx, _data: &mut AppState, _env| {
            let cmd = Command::new(START_SLOW_FUNCTION, 0);
            ctx.submit_command(cmd, None);
        })
        .padding(5.0);
    let button_placeholder = Label::new(LocalizedString::new("Processing..."))
        .padding(5.0)
        .center();

    let text = LocalizedString::new("hello-counter")
        .with_arg("count", |data: &AppState, _env| (data.value).into());
    let label = Label::new(text).padding(5.0).center();

    let either = Either::new(|data, _env| data.processing, button_placeholder, button);

    Flex::column().with_child(label).with_child(either)
}
fn main() {
    let main_window =
        WindowDesc::new(|| ui_builder()).title(LocalizedString::new("Blocking functions"));
    let app = AppLauncher::with_window(main_window);
    let delegate = Delegate {
        eventsink: app.get_external_handle(),
    };
    app.delegate(delegate)
        .use_simple_logger()
        .launch(AppState::default())
        .expect("launch failed");
}
