# 0.2.0

## Added

- `<details>` / `<summary>` dropdowns

## Changed

- iced's `wgpu` and `tiny-skia` backends can now be toggled through `iced-*` backend crate features

### State updating

- `MarkWidget::on_updating_state` now takes in a `Fn(UpdateMsg) -> Message`.
- `MarkState::update` now takes in a `UpdateMsg`.
- Wrap `UpdateMsg` in your message type and pass it to the update function.
- This was done to support dropdowns and clean up code

Example:

```diff
  MarkWidget::new(&self.state)
-     .on_updating_state(|| Message::UpdateState)
+     .on_updating_state(|action| Message::UpdateState(action))
```

```diff
- Message::UpdateState => self.state.update(),
+ Message::UpdateState(action) => self.state.update(action),
```

```diff
  enum Message {
-     UpdateState,
+     UpdateState(frostmark::UpdateMsg),
      // ...
  }
```

## Fixed

- Space/whitespace issues in formatted text
- Broken details/summary handling
