---
name: rust-iced
description: >-
  Build and debug Iced 0.14 GUI code in Rust (edition 2024). Use when writing
  or modifying Iced widgets, application state, Message/update/view logic,
  subscriptions, async Tasks, theming, or custom layouts in this workspace.
---

# Rust + Iced 0.14

This workspace targets **Iced 0.14**, Rust edition 2024 (MSRV 1.85). Iced 0.14
uses the **functional application API** — there is no `Application`/`Sandbox`
trait. Do not use pre-0.13 patterns (`Application::run`, `iced::Settings`,
`Command`) — they are removed.

## Core architecture (TEA)

State + `update(state, message) -> Task<Message>` + `view(state) -> Element`.

```rust
use iced::{Element, Task};

#[derive(Debug, Clone)]
enum Message {
    Increment,
    Loaded(Result<Data, String>),
}

#[derive(Default)]
struct State { count: i64 }

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::Increment => { state.count += 1; Task::none() }
        Message::Loaded(res) => { /* ... */ Task::none() }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    use iced::widget::{button, column, text};
    column![
        text(state.count),
        button("+").on_press(Message::Increment),
    ].into()
}
```

## Launching

```rust
fn main() -> iced::Result {
    iced::application("Title", update, view)
        // optional builders, in any order:
        .theme(|_state| iced::Theme::Dark)
        .subscription(subscription)
        .window_size((800.0, 600.0))
        .run()
}
```

- `iced::application(title, update, view)` is the entry. `title` can be a
  `&str`, `String`, or `Fn(&State) -> String`.
- For state that needs async init: `.run_with(|| (State::new(), Task::perform(...)))`.
- `iced::run(update, view)` is the minimal form (no title/builders).

## Async work — `Task`, not `Command`

- `Task::none()` — do nothing.
- `Task::perform(future, Message::Variant)` — run a future, map output to a message.
- `Task::done(msg)` — emit a message immediately.
- `Task::batch([t1, t2])` — combine.
- Chain with `.then(...)`, transform with `.map(...)`.
- `tokio` is enabled (feature `tokio`), so async fns work directly inside `Task::perform`.

```rust
Task::perform(load_data(path), Message::Loaded)
```

## Subscriptions (events over time)

```rust
fn subscription(_state: &State) -> iced::Subscription<Message> {
    iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick)
}
```

Use `iced::keyboard::on_key_press(...)`, `iced::event::listen_with(...)`, or
`Subscription::run(...)` for custom streams. Wire via `.subscription(subscription)`.

## Widgets & layout

- Macros: `row![]`, `column![]`, `text!("{}", x)` (formatting macro).
- Functions: `button`, `text`, `text_input`, `checkbox`, `slider`, `pick_list`,
  `scrollable`, `container`, `image`, `svg`, `space`, `stack`, `tooltip`.
- Sizing: `.width(Fill)` / `.height(Shrink)` — import `iced::Length::{Fill, Shrink, Fixed}` or use `Fill`/`Shrink` from `iced` prelude.
- Spacing/padding: `column![].spacing(10).padding(20)`.
- `image` and `svg` features are enabled — `image(handle)`, `svg(handle)` are available.

## `Element` lifetimes

`view` returns `Element<'a, Message>` borrowing from `&'a State`. Build child
widgets inline or keep borrows alive; `.into()` converts a concrete widget to
`Element`. Avoid storing `Element` in state.

## Conventions for this workspace

- GUI lives in `crates/librapix-app`. Core logic is in sibling crates
  (`librapix-core`, `-indexer`, `-search`, etc.) — keep `update`/`view` thin and
  delegate heavy work to those crates via `Task::perform`.
- i18n strings come from `librapix-i18n` — do not hardcode user-facing text.
- Run checks with `cargo clippy --workspace --all-targets` and
  `cargo fmt`. The repo lints strictly — fix all clippy warnings.

## Gotchas

- `Message` must be `Clone` (and usually `Debug`). Wrap non-Clone async results
  in `Result<_, String>` or `Arc<_>`.
- No blocking I/O in `update`/`view` — always offload to `Task::perform`.
- Theme/style closures take `&State` (for theme) or `&Theme` (for widget style fns).
- When in doubt about an API, check docs.rs/iced/0.14 — the API moved fast across
  0.12 → 0.13 → 0.14; older Stack Overflow answers are usually wrong.
