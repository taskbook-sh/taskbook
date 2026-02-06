# Plan: Improve Board Selection UX

## Problem
The "Move to board" operation (`m` key) requires typing the board name manually, which is cumbersome. Users should always select boards from a list, with an easy way to create new boards.

## Current State
- **Creating tasks/notes (`t`/`n`)**: Uses board picker list - GOOD
- **Moving items (`m`)**: Requires typing board name - BAD
- **Creating boards (`B`)**: Requires typing - OK (necessary for new names)

## Proposed Solution

### Change 1: Replace text input with board picker for "Move to board"

**In `app.rs`:**
- Replace `MoveBoard { id, input, cursor }` with `SelectBoardForMove { id, selected }`

**In `actions.rs`:**
- Change `m` key handler to open `SelectBoardForMove` popup instead of `MoveBoard`
- Add handler for `SelectBoardForMove` that:
  - Navigates with `j`/`k` or arrow keys
  - On Enter: moves item to selected board (or opens CreateBoard if "New board" selected)
  - On Esc: cancels

**In `ui.rs`:**
- Render `SelectBoardForMove` using existing `render_board_picker` with title "Move to Board"

### Change 2: Track pending action when creating a new board

When user selects "+ New board..." during move/task/note creation, we need to remember what action to continue with after the board is created.

**In `app.rs`:**
- Add a new popup state or modify `CreateBoard` to track the pending action:
```rust
CreateBoard {
    input: String,
    cursor: usize,
    then: Option<PendingBoardAction>,
}

enum PendingBoardAction {
    CreateTask,
    CreateNote,
    MoveItem { id: u64 },
}
```

**In `actions.rs`:**
- When CreateBoard completes, check `then` field and transition to the appropriate next state

## Files to Modify
1. `src/tui/app.rs` - Update `PopupState` enum
2. `src/tui/actions.rs` - Update key handlers and add new popup handler
3. `src/tui/ui.rs` - Add rendering for new popup state

## Summary
This change ensures users never need to type a board name unless creating a genuinely new board. The flow becomes:
1. Press `m` to move → see list of boards + "New board..."
2. Select existing board → item moves immediately
3. Select "New board..." → type name → item moves to new board
