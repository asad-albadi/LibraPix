use iced::time;
use iced_futures::backend::default::Executor;
use iced_futures::futures::StreamExt;
use iced_futures::futures::channel::mpsc;
use iced_futures::futures::stream;
use iced_futures::subscription;
use iced_futures::{Runtime, Subscription};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestMessage {
    BatchComplete,
    UpdateTick,
    SnapshotTick,
    DeferredTick,
    ReconcileTick,
}

fn timer_subscription(interval: Duration, message: TestMessage) -> Subscription<TestMessage> {
    time::every(interval)
        .with(message)
        .map(|(message, _instant)| message)
}

#[test]
fn active_timer_subscriptions_do_not_delay_runtime_stream_messages() {
    let executor = <Executor as iced_futures::Executor>::new().expect("create Iced executor");
    let (sender, mut receiver) = mpsc::channel(32);
    let mut runtime = Runtime::new(executor, sender);

    runtime.track(subscription::into_recipes(Subscription::batch(vec![
        timer_subscription(Duration::from_secs(1), TestMessage::UpdateTick),
        timer_subscription(Duration::from_secs(1), TestMessage::SnapshotTick),
        timer_subscription(Duration::from_secs(1), TestMessage::DeferredTick),
        timer_subscription(Duration::from_secs(1), TestMessage::ReconcileTick),
    ])));

    let started_at = Instant::now();
    runtime.run(stream::once(async { TestMessage::BatchComplete }).boxed());

    let (received_at, message) = runtime.block_on(async {
        let message = receiver
            .next()
            .await
            .expect("runtime should deliver a message");
        (Instant::now(), message)
    });

    assert_eq!(message, TestMessage::BatchComplete);
    assert!(
        received_at.duration_since(started_at) < Duration::from_millis(150),
        "background completion message should not wait behind timer subscriptions",
    );
}
