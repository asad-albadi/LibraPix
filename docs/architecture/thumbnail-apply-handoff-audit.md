# Thumbnail Apply Handoff Audit

Date: 2026-03-10

Scope: the remaining Windows large-library issue where thumbnail worker batches finished quickly but the UI reflected the results much later.

## Primary evidence

Observed logs:

- `logs/librapix-startup-20260310-173448-29500.log`
- `logs/librapix-startup-20260310-174212-30216.log`

Key timings:

- In `173448`, `startup.ready` was reached at `+1913ms`.
- In the same log, the startup-priority image batch ended at `+3886ms`:
  - `startup.thumbnail.batch.end | generation=1 batch_id=1 mode=startup_priority ... elapsed_ms=1972`
- The corresponding apply did not happen until `+163590ms`:
  - `startup.thumbnail.apply | generation=1 batch_id=1 mode=startup_priority ...`
- That is a `159704ms` completion-to-apply gap even though the worker itself only took `1972ms`.

Control evidence from the same code path:

- In `173448`, the later background-catchup batches apply within `9ms..25ms` of `startup.thumbnail.batch.end`.
- In `174212`, background-catchup batches apply within `10ms..217ms` of `startup.thumbnail.batch.end`.

Conclusion:

- `apply_thumbnail_batch_result(...)` itself was not the bottleneck.
- The bad delay sat before `Message::ThumbnailBatchComplete` reached `update`.

## Exact code path

The handoff path is:

1. `run_next_thumbnail_batch_if_idle(...)`
2. `Task::perform(async move { do_thumbnail_batch(input) }, ...)`
3. Iced runtime forwards the completion message
4. `Message::ThumbnailBatchComplete(result)`
5. `apply_thumbnail_batch_result(...)`
6. `patch_thumbnail_paths(...)` updates gallery/timeline/search items

Relevant code:

- `crates/librapix-app/src/main.rs`
  - `subscription(...)`
  - `run_next_thumbnail_batch_if_idle(...)`
  - `do_thumbnail_batch(...)`
  - `apply_thumbnail_batch_result(...)`

Important negative finding:

- `apply_thumbnail_batch_result(...)` does not defer thumbnail application behind later projection, reconcile, or extra coalescing.
- The generation-stale path only drops mismatched generations immediately.
- Therefore the large gap could not be explained by app-side batching or deferred apply logic.

## Root cause

The real cause was the timer-subscription implementation in `subscription(...)`.

Before this fix, the app created periodic subscriptions with:

- `Subscription::run(...)`
- `iced::stream::channel(...)`
- `std::thread::sleep(...)` inside async loops

That pattern was used for:

- update-check tick
- startup-reconcile tick
- snapshot-apply tick
- deferred-thumbnail tick

Why this was wrong:

- Librapix was still using Iced's default native executor selection, which falls back to the thread-pool backend when `tokio`/`smol` are not enabled.
- The custom timer loops were blocking sleeps inside futures running on the Iced runtime.
- Blocking executor threads is the wrong shape for async timer subscriptions and can delay unrelated runtime message forwarding.

Why the logs fit this explanation:

- The delayed batch was not stale and had no pending projection/reconcile gate.
- The worker completed quickly, but no `ThumbnailBatchComplete` was applied until a much later wake.
- Other batches applied promptly once the runtime was active again.

## Online research result

Official Iced guidance favors `iced::time::every(...)` for periodic subscriptions.

Relevant findings:

- Iced documents `Subscription` as the declarative way to listen to streams such as time.
- `iced::time::every(...)` is the supported timer API.
- Iced's default backend selects `tokio`, then `smol`, then `thread-pool`.
- On the tokio backend, `time::every(...)` is backed by `tokio::time::interval`.
- Rust documents `std::thread::sleep` as blocking the current thread.

That combination makes the fix straightforward and industry-standard:

- stop building timer subscriptions from blocking sleep loops
- use Iced's timer API
- use the supported async timer backend so timers do not occupy executor workers

## Fix implemented

Implemented on this branch:

1. Enabled Iced's `tokio` feature in the workspace dependency configuration.
2. Replaced the four custom `Subscription::run(... std::thread::sleep ...)` tick streams with `iced::time::every(...)`.
3. Added explicit thumbnail handoff logs:
   - `startup.thumbnail.batch.dispatch_to_ui`
   - `startup.thumbnail.batch.message_received`
   - `startup.thumbnail.apply.start`
   - existing `startup.thumbnail.apply` now also records handoff delays
   - `startup.thumbnail.batch.handoff.slow` when completion-to-receive delay is unexpectedly high
4. Added a regression test exercising the Iced runtime with active timer subscriptions and verifying prompt stream-message delivery:
   - `crates/librapix-app/tests/runtime_message_handoff.rs`

## Final policy

What must happen now:

- worker completion should dispatch to the UI immediately
- `ThumbnailBatchComplete` should be applied promptly once delivered
- timer subscriptions must never occupy executor threads with blocking sleeps
- any future completion-to-apply delay must be visible in logs as:
  - worker finish
  - dispatch to UI
  - message received
  - apply start/end

## Remaining caveat

This fix removes the identified runtime handoff bug and adds proof-oriented logging, but the final product judgment still needs one manual Windows large-library GUI pass to confirm the feel is materially corrected on the user machine.
