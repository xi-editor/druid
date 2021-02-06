# Examples

There are some diferent kind of examples, some specifically show of one 
Druid concept, some are tools used for testing and debugging, and
others are more complete examples of how to tie everything together.
The later are listed separately under [showcases](##Showcases).

## Anim
```
cargo run --example anim
```
This example shows how to make a simple animation using `Event::AnimFrame`.

## Async Event
```
cargo run --example async_event
```
Having a long function running in the background feeding in new data is important to not block the UI from running. This is similair to [blocking function](##Blocking Function) but here the function runs for the lifetime of the program. 

## Blocking Functions
```
cargo run --example blocking_functions
```
Sometimes you need to fetch some data from disk or from the internet, but you should never block the UI thread with long running operations! Instead you should spawn of a new thread (or something liks async functions) and have it send you the data back like here. This is very similair to [async event](##Async Event) except the lifetime of the thread is shorter here.

## Cursor
```
cargo run --example cursor
```
This example demonstrates how to set the cursor icon, and how to use a custom cursor.

## Custom Widget
```
cargo run --example custom_widget
```
This shows how to use all of the methods on `PaintCtx` used for drawing on a canvas.
You can use this to draw everything from text to images to curves.
## Either
```
cargo run --example either
```
This example shows how to use the `Either` widget, which shows one of two children based on some predicate.
This can be useful for loading screens, error messages, and other situations where you want to show one of
two possible widgets.

## Hello
```
cargo run --example hello
```
This shows some of the basics of druid. If you need a start of how to build an application with a text-box and some labels this is where to start.

## Hello_web
```
cd druid/examples/hello_web
wasm-pack build --out-dir pkg --out-name hello_web
```
This is an example of how to get almost any druid application can be used on the web. This is just the hello_world example but should work for all of them.

## Identity
```
cargo run --example identity
```
In druid identity is used to send specific widgets commands. Instead of a command going to all the widgets, you can send them to just the one you need. This example has some colorwels and some buttons that interact with them. All of them are identical, except the identity, which makes it possible for the buttons to only affect a single colorwel.

## Invalidation
```
cargo run --example invalidation --features="im"
```
A demonstration how to use debug invalidation regeons in your own widgets, including some examples of builtin widgets.

## Layout
```
cargo run --example layout
```
An example of how basic widget composition works in druid. There are no custom widgets just compositions of builtin ones.

## Lens
```
cargo run --example lens
```
Lenses are a core part of druid, they allow you to zoom into a part of the app state.

## List
```
cargo run --example list --features="im"
```
This shows you how you could, for example, add items to lists and delete them. 

## Multiwine
```
cargo run --example multiwin
```
Having multiple windows is a super nice tool to have when developing applications. This shows you the basic setup you need for a second window.

## Open Save
```
cargo run --example open_save
```
Opening and saving files is crucial for a lot of applications. This shows you how to get opening and saving files working cross platform.

## Panels
```
cargo run --example panels
```
Very similair to [layout](##Layout) but it splits the screen into 2 segments

## Parse
```
cargo run --example parse
```
Druid doesnt have numeric specific texboxes, instead you have to parse the input as if it ware a numeric value. 

## Scroll
```
cargo run --example scroll
```
Scrolling is a great way to show more content then can be displayed on the screen at a time. This is an example showing you how to use them.

## Split
```
cargo run --example split_demo
```
An example of how to split a widget in 2 in various ways. This also includes having the user drag the border!! 

## Styled_text
```
cargo run --example styled_text
```
Not all text should look the same. You are able to change a lot of things, color, size, and monospace. This example shows how ot change these propperties.

## Svg
```
cargo run --example svg
```
This shows you how to display an SVG as a widget.

## Switches
```
cargo run --example switches
```
Switches are usefullin many ways, this example shows how to use the druid built-in ones. This includes on/off and up/down for incrementing numeric values.

## Tabs
```
cargo run --example tabs
```
Tabs allow you to seperate diferent portions of the UI. This example shows you how to use them in druid. Similair to [view switcher](##View Switcher) but with with a diferent purpose.

## Timers
```
cargo run --example timers
```
Timers allow you to send events to your widgets at a certain points inthe future. This example shows how to use them.

## View Switcher
```
cargo run --example view_switcher
```
Very smilair to [tabs](##Tabs) but this allows you to have more controll over it. This allows you to switch out widgets on the fly.

## Showcases

### Calc
```
cargo run --example calc
```

This is a showcase of a simple calculator. There are better ways to implement the calculator logic, but it provides all the standard operations like adition devision multiplication C and CE.

### Event Viewer
```
cargo run --example event_viewer
```

This shows you how to capture most events and you can see what gives which event with what data. You can compare mouse clicks to keyboard typing and see how modifiers, like shift and ctrl, are handled.

### Flex
```
cargo run --example flex
```

Flex shows off all the things you can do with flex elements. You can play with all the setings and it will change in real-time.

### Game Of Life
```
cargo run --example game_of_life
```

A simple implementation of conway's game of life. You can change the evolution speed, and pauze so you can take your time making your own creations!

### Image
```
cargo run --example image
```

Image shows off all the knobs you can turn on images. You can play with them with real time results, which you to fogure out what settings are best for you.

Please note that the image is exported with some kind of interpolation. So even when you turn interpolation off/NearestNeighbor in druid, you will still see this because thats how the image actually looks.

### Scroll Colors
```
cargo run --example scroll_colors
```

This is a showcase is scrolling through an image gradient square. The square is devided into smaller squares each with a unique color. There are other ways to to this like one big widget with an image for example.

### Widget Gallery
```
cargo run --example widget_gallery
```

This is a showcase of some simple widgets with their default styling. These are interactive, but you cannot change any of their styling.