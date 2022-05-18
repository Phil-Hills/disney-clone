use smallvec::{smallvec, SmallVec};
use tracing::{trace_span, Span};

use widget_cruncher::promise::PromiseToken;
use widget_cruncher::shell::keyboard_types::Key;
use widget_cruncher::widget::prelude::*;
use widget_cruncher::widget::{AsWidgetPod, ClipBox, Flex, Spinner, WidgetPod};
use widget_cruncher::{Color, Command, Point, Selector, Target};

use crate::content_set::{ContentSet, ContentSetMetadata};
use crate::thumbnail::CHANGE_SELECTED_ITEM;

const REQUEST_FOCUS: Selector = Selector::new("request_focus");

// Loads and parses https://cd-static.bamgrid.com/dp-117731241344/home.json
fn load_collection(url: &str) -> Result<Vec<ContentSetMetadata>, reqwest::Error> {
    let json: serde_json::Value = reqwest::blocking::get(url)?.json()?;
    let containers = json["data"]["StandardCollection"]["containers"].clone();
    let container_items = containers
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|container| {
            let title = container["set"]["text"]["title"]["full"]["set"]["default"]["content"]
                .as_str()?
                .to_string();
            let ref_id = container["set"]["refId"].as_str()?.to_string();
            Some(ContentSetMetadata { title, ref_id })
        })
        .collect::<Vec<_>>();
    Ok(container_items)
}

pub struct RootWidget {
    // The promise token is mostly a type-system aid to "prove" to the compiler
    // that the result you're getting is the same you asked for earlier.
    pub children_promise: PromiseToken<Vec<ContentSetMetadata>>,

    // What's we're actually displaying.
    pub children: WidgetPod<ClipBox<Flex>>,

    // A very bare-bones "cursor" sent to every child to
    // choose which one should have the "is selected" outline and big size.
    pub selected_item: (usize, usize),
}

impl RootWidget {
    pub fn new() -> Self {
        let placeholder = Spinner::new();
        let column = Flex::column().with_child(placeholder);
        let clipbox = ClipBox::new(column).constrain_horizontal(true);
        Self {
            children_promise: PromiseToken::empty(),
            children: WidgetPod::new(clipbox),
            selected_item: (0, 0),
        }
    }
}

// --- TRAIT IMPL ---

impl Widget for RootWidget {
    fn on_event(&mut self, ctx: &mut EventCtx, event: &Event, env: &Env) {
        ctx.init();
        match event {
            // This happens after the callback passed to `ctx.compute_in_background` returns
            Event::PromiseResult(result) => {
                if let Some(children) = result.try_get(self.children_promise) {
                    // TODO - Need to find a more idiomatic way to do this.
                    self.children.recurse_pass(
                        "custom_pass",
                        &mut ctx.widget_state,
                        // clipbox is an alias of self.children in this closure
                        |clipbox, clipbox_state| {
                            clipbox.child.recurse_pass(
                                "custom_pass",
                                clipbox_state,
                                |flex, flex_state| {
                                    flex.clear(flex_state);
                                    for (row, child) in children.into_iter().enumerate() {
                                        flex.add_child(flex_state, ContentSet::new(row, child));
                                        flex.add_spacer(flex_state, 30.0);
                                    }
                                    // when this closure returns, the framework automatically merges
                                    // invalidated state
                                },
                            );
                        },
                    );

                    ctx.skip_child(&mut self.children);
                    return;
                }
            }
            Event::KeyDown(key_event) => {
                // This is a HUGE cheat.
                match &key_event.key {
                    Key::ArrowDown => {
                        self.selected_item.0 = self.selected_item.0.saturating_add(1);
                    }
                    Key::ArrowLeft => {
                        self.selected_item.1 = self.selected_item.1.saturating_sub(1);
                    }
                    Key::ArrowRight => {
                        self.selected_item.1 = self.selected_item.1.saturating_add(1);
                    }
                    Key::ArrowUp => {
                        self.selected_item.0 = self.selected_item.0.saturating_sub(1);
                    }
                    _ => {}
                }

                ctx.submit_command(CHANGE_SELECTED_ITEM.with(self.selected_item));
            }
            Event::Command(command) if command.is(REQUEST_FOCUS) => {
                ctx.request_focus();
            }
            _ => {}
        }
        self.children.on_event(ctx, event, env)
    }

    fn on_status_change(&mut self, _ctx: &mut LifeCycleCtx, _event: &StatusChange, _env: &Env) {}

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, env: &Env) {
        const COLLECTION_URL: &str = "https://cd-static.bamgrid.com/dp-117731241344/home.json";

        ctx.init();
        match event {
            // This is a bit of a hack: first RootWidget registers as able to receive events,
            // then it sends a Command to itself so that it can request_focus(). Requesting
            // focus in necessary to get keyboard events.
            // This is unnecessarily complicated; probably should change Druid's architecture
            // to make it simpler.
            LifeCycle::BuildFocusChain => {
                ctx.register_for_focus();
                ctx.submit_command(
                    Command::from(REQUEST_FOCUS).to(Target::Widget(ctx.widget_id())),
                );
            }
            // This is essentially a second constructor.
            LifeCycle::WidgetAdded => {
                self.children_promise =
                    ctx.compute_in_background(move |_| load_collection(COLLECTION_URL).unwrap());
            }
            _ => {}
        }
        self.children.lifecycle(ctx, event, env)
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, env: &Env) -> Size {
        let layout = self.children.layout(ctx, bc, env);
        self.children.set_origin(ctx, env, Point::ORIGIN);
        layout
    }

    fn paint(&mut self, ctx: &mut PaintCtx, env: &Env) {
        ctx.init();

        const BACKGROUND_COLOR: Color = Color::from_rgba32_u32(0x07_1b_0f_ff);

        let paint_rect = ctx.size().to_rect();
        ctx.fill(paint_rect, &BACKGROUND_COLOR);
        self.children.paint(ctx, env)
    }

    fn children(&self) -> SmallVec<[&dyn AsWidgetPod; 16]> {
        smallvec![&self.children as &dyn AsWidgetPod]
    }

    fn children_mut(&mut self) -> SmallVec<[&mut dyn AsWidgetPod; 16]> {
        smallvec![&mut self.children as &mut dyn AsWidgetPod]
    }

    // This isn't useful for the application itself, but it makes traces more readable
    // when debugging
    fn make_trace_span(&self) -> Span {
        trace_span!("RootWidget")
    }
}
