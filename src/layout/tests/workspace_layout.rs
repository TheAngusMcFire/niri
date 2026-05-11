use niri_ipc::{ColumnDisplay, ColumnWindowHeight, ColumnWidthLayout, WorkspaceLayoutTree};

use super::*;

// ── helpers ──────────────────────────────────────────────────────────────────

fn make_layout_with_output() -> Layout<TestWindow> {
    let mut layout = Layout::default();
    Op::AddOutput(1).apply(&mut layout);
    layout
}

fn add_tiling(layout: &mut Layout<TestWindow>, id: usize) {
    Op::AddWindow {
        params: TestWindowParams::new(id),
    }
    .apply(layout);
}

fn add_floating(layout: &mut Layout<TestWindow>, id: usize) {
    Op::AddWindow {
        params: TestWindowParams::new(id),
    }
    .apply(layout);
    Op::SetWindowFloating {
        id: Some(id),
        floating: true,
    }
    .apply(layout);
}

fn get_tree(layout: &Layout<TestWindow>) -> WorkspaceLayoutTree {
    layout.workspace_layout_tree(None).expect("active workspace")
}

fn apply_tree(layout: &mut Layout<TestWindow>, tree: WorkspaceLayoutTree) {
    layout
        .apply_workspace_layout_tree(None, tree)
        .expect("apply succeeded");
    layout.verify_invariants();
}

fn apply_tree_err(layout: &mut Layout<TestWindow>, tree: WorkspaceLayoutTree) -> String {
    layout
        .apply_workspace_layout_tree(None, tree)
        .expect_err("expected an error")
}

fn column_ids(tree: &WorkspaceLayoutTree) -> Vec<Vec<u64>> {
    tree.columns
        .iter()
        .map(|col| col.windows.iter().map(|w| w.window_id).collect())
        .collect()
}

fn floating_ids(tree: &WorkspaceLayoutTree) -> Vec<u64> {
    tree.floating.iter().map(|e| e.window_id).collect()
}

// ── GET: layout_tree ─────────────────────────────────────────────────────────

#[test]
fn layout_tree_empty_workspace() {
    let layout = make_layout_with_output();
    let tree = get_tree(&layout);

    assert!(tree.columns.is_empty());
    assert!(tree.floating.is_empty());
    assert_eq!(tree.active_column_idx, 0);
}

#[test]
fn layout_tree_single_tiling_window() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let tree = get_tree(&layout);

    assert_eq!(column_ids(&tree), vec![vec![1]]);
    assert!(tree.floating.is_empty());
    assert_eq!(tree.active_column_idx, 0);
    assert_eq!(tree.columns[0].active_window_idx, 0);
}

#[test]
fn layout_tree_two_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let tree = get_tree(&layout);

    assert_eq!(column_ids(&tree), vec![vec![1], vec![2]]);
    assert_eq!(tree.active_column_idx, 1);
}

#[test]
fn layout_tree_stacked_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let tree = get_tree(&layout);

    assert_eq!(column_ids(&tree), vec![vec![1, 2]]);
    assert_eq!(tree.columns[0].active_window_idx, 1);
}

#[test]
fn layout_tree_active_column_idx() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    add_tiling(&mut layout, 3);
    Op::FocusColumnLeft.apply(&mut layout);

    let tree = get_tree(&layout);

    assert_eq!(tree.active_column_idx, 1);
}

#[test]
fn layout_tree_floating_window() {
    let mut layout = make_layout_with_output();
    add_floating(&mut layout, 1);

    let tree = get_tree(&layout);

    assert!(tree.columns.is_empty());
    assert_eq!(floating_ids(&tree), vec![1]);
}

#[test]
fn layout_tree_mixed_tiling_and_floating() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_floating(&mut layout, 2);

    let tree = get_tree(&layout);

    assert_eq!(column_ids(&tree), vec![vec![1]]);
    assert_eq!(floating_ids(&tree), vec![2]);
}

#[test]
fn layout_tree_column_width_preserved() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    // Without a configured default_column_width the window's own width is used (Fixed).
    let tree = get_tree(&layout);
    assert!(matches!(
        tree.columns[0].width,
        ColumnWidthLayout::Fixed(_)
    ));
}

#[test]
fn layout_tree_display_mode_preserved() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);
    Op::ToggleColumnTabbedDisplay.apply(&mut layout);

    let tree = get_tree(&layout);

    assert_eq!(tree.columns[0].display, ColumnDisplay::Tabbed);
}

// ── SET: apply_layout_tree (valid) ───────────────────────────────────────────

#[test]
fn apply_layout_tree_round_trip_single_window() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree.clone());

    assert_eq!(column_ids(&get_tree(&layout)), column_ids(&tree));
}

#[test]
fn apply_layout_tree_round_trip_two_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree.clone());

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![1], vec![2]]);
}

#[test]
fn apply_layout_tree_round_trip_stacked_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree.clone());

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![1, 2]]);
}

#[test]
fn apply_layout_tree_round_trip_floating() {
    let mut layout = make_layout_with_output();
    add_floating(&mut layout, 1);

    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree.clone());

    assert_eq!(floating_ids(&get_tree(&layout)), vec![1]);
    assert!(get_tree(&layout).columns.is_empty());
}

#[test]
fn apply_layout_tree_reorder_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    tree.columns.reverse();
    apply_tree(&mut layout, tree);

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![2], vec![1]]);
}

#[test]
fn apply_layout_tree_reorder_windows_in_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows.reverse();
    apply_tree(&mut layout, tree);

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![2, 1]]);
}

#[test]
fn apply_layout_tree_merge_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    // Move window 2 into column 0.
    let win2 = tree.columns[1].windows.remove(0);
    tree.columns[0].windows.push(win2);
    tree.columns.remove(1);
    apply_tree(&mut layout, tree);

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![1, 2]]);
}

#[test]
fn apply_layout_tree_split_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let mut tree = get_tree(&layout);
    // Move window 2 into its own new column.
    let win2 = tree.columns[0].windows.remove(1);
    tree.columns.push(niri_ipc::WorkspaceColumn {
        windows: vec![win2],
        active_window_idx: 0,
        width: ColumnWidthLayout::Proportion(0.5),
        is_full_width: false,
        display: ColumnDisplay::Normal,
    });
    apply_tree(&mut layout, tree);

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![1], vec![2]]);
}

#[test]
fn apply_layout_tree_move_window_between_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    add_tiling(&mut layout, 3);

    let mut tree = get_tree(&layout);
    // Move window 3 into column 0 (with window 1).
    let win3 = tree.columns[2].windows.remove(0);
    tree.columns[0].windows.push(win3);
    tree.columns.remove(2);
    apply_tree(&mut layout, tree);

    assert_eq!(column_ids(&get_tree(&layout)), vec![vec![1, 3], vec![2]]);
}

#[test]
fn apply_layout_tree_tiling_to_floating() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    // Move window 2 from tiling to floating.
    let win2_id = tree.columns.remove(1).windows.remove(0).window_id;
    tree.floating.push(niri_ipc::FloatingWindowEntry {
        window_id: win2_id,
        position: (100., 100.),
    });
    apply_tree(&mut layout, tree);

    let result = get_tree(&layout);
    assert_eq!(column_ids(&result), vec![vec![1]]);
    assert_eq!(floating_ids(&result), vec![2]);
}

#[test]
fn apply_layout_tree_floating_to_tiling() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_floating(&mut layout, 2);

    let mut tree = get_tree(&layout);
    // Move window 2 from floating to a new tiling column.
    let float2 = tree.floating.remove(0);
    tree.columns.push(niri_ipc::WorkspaceColumn {
        windows: vec![niri_ipc::ColumnWindow {
            window_id: float2.window_id,
            height: ColumnWindowHeight::Auto { weight: 1.0 },
        }],
        active_window_idx: 0,
        width: ColumnWidthLayout::Proportion(0.5),
        is_full_width: false,
        display: ColumnDisplay::Normal,
    });
    apply_tree(&mut layout, tree);

    let result = get_tree(&layout);
    assert_eq!(column_ids(&result), vec![vec![1], vec![2]]);
    assert!(floating_ids(&result).is_empty());
}

#[test]
fn apply_layout_tree_change_column_width() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].width = ColumnWidthLayout::Fixed(400.0);
    apply_tree(&mut layout, tree);

    let result = get_tree(&layout);
    assert_eq!(result.columns[0].width, ColumnWidthLayout::Fixed(400.0));
}

#[test]
fn apply_layout_tree_change_tile_height() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows[0].height = ColumnWindowHeight::Fixed(200.0);
    apply_tree(&mut layout, tree);

    let result = get_tree(&layout);
    assert_eq!(
        result.columns[0].windows[0].height,
        ColumnWindowHeight::Fixed(200.0)
    );
}

#[test]
fn apply_layout_tree_change_active_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    tree.active_column_idx = 0;
    apply_tree(&mut layout, tree);

    assert_eq!(get_tree(&layout).active_column_idx, 0);
}

#[test]
fn apply_layout_tree_active_column_clamped() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    tree.active_column_idx = 999;
    apply_tree(&mut layout, tree);

    // Should be clamped to the last valid column.
    assert_eq!(get_tree(&layout).active_column_idx, 1);
}

#[test]
fn apply_layout_tree_active_window_clamped() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let mut tree = get_tree(&layout);
    tree.columns[0].active_window_idx = 999;
    apply_tree(&mut layout, tree);

    // Should be clamped to the last valid tile.
    assert_eq!(get_tree(&layout).columns[0].active_window_idx, 1);
}

#[test]
fn apply_layout_tree_three_windows_permutation() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    add_tiling(&mut layout, 3);

    let mut tree = get_tree(&layout);
    // Rotate: [1][2][3] → [3][1][2]
    tree.columns.rotate_right(1);
    apply_tree(&mut layout, tree);

    assert_eq!(
        column_ids(&get_tree(&layout)),
        vec![vec![3], vec![1], vec![2]]
    );
}

#[test]
fn apply_layout_tree_is_full_width_preserved() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].is_full_width = true;
    apply_tree(&mut layout, tree);

    assert!(get_tree(&layout).columns[0].is_full_width);
}

// ── SET: apply_layout_tree (validation errors) ───────────────────────────────

#[test]
fn apply_layout_tree_error_empty_column() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows.clear();

    let err = apply_tree_err(&mut layout, tree);
    assert!(err.contains("no windows"), "unexpected error: {err}");
}

#[test]
fn apply_layout_tree_error_duplicate_id_in_columns() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    // Make column 1 also reference window 1.
    tree.columns[1].windows[0].window_id = 1;

    let err = apply_tree_err(&mut layout, tree);
    assert!(err.contains("more than once"), "unexpected error: {err}");
}

#[test]
fn apply_layout_tree_error_duplicate_id_across_tiling_and_floating() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    let mut tree = get_tree(&layout);
    // Add window 1 also to floating.
    tree.floating.push(niri_ipc::FloatingWindowEntry {
        window_id: 1,
        position: (0., 0.),
    });

    let err = apply_tree_err(&mut layout, tree);
    assert!(err.contains("more than once"), "unexpected error: {err}");
}

#[test]
fn apply_layout_tree_error_unknown_window_id() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows[0].window_id = 999;

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("not on workspace"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_error_missing_window() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);

    // Submit only window 1, omitting window 2.
    let mut tree = get_tree(&layout);
    tree.columns.remove(1);

    let err = apply_tree_err(&mut layout, tree);
    assert!(err.contains("missing from"), "unexpected error: {err}");
}

#[test]
fn apply_layout_tree_error_negative_auto_weight() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    Op::ConsumeOrExpelWindowLeft { id: None }.apply(&mut layout);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows[0].height = ColumnWindowHeight::Auto { weight: -1.0 };

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("weight must be positive"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_error_zero_auto_weight() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows[0].height = ColumnWindowHeight::Auto { weight: 0.0 };

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("weight must be positive"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_error_negative_fixed_height() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].windows[0].height = ColumnWindowHeight::Fixed(-100.0);

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("Fixed height must be positive"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_error_negative_proportion_width() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].width = ColumnWidthLayout::Proportion(-0.5);

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("Proportion must be positive"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_error_zero_fixed_width() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);

    let mut tree = get_tree(&layout);
    tree.columns[0].width = ColumnWidthLayout::Fixed(0.0);

    let err = apply_tree_err(&mut layout, tree);
    assert!(
        err.contains("Fixed must be positive"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_layout_tree_empty_workspace_is_noop() {
    let mut layout = make_layout_with_output();

    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree);

    let result = get_tree(&layout);
    assert!(result.columns.is_empty());
    assert!(result.floating.is_empty());
}

fn view_pos(layout: &Layout<TestWindow>) -> f64 {
    layout
        .active_workspace()
        .expect("active workspace")
        .scrolling()
        .view_pos()
}

#[test]
fn apply_layout_tree_round_trip_preserves_view_pos() {
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    add_tiling(&mut layout, 3);

    let before = view_pos(&layout);
    let tree = get_tree(&layout);
    apply_tree(&mut layout, tree);
    let after = view_pos(&layout);

    assert!(
        (before - after).abs() < 1e-6,
        "view_pos changed across round-trip: {before} -> {after}"
    );
}

#[test]
fn apply_layout_tree_change_active_column_preserves_view_pos() {
    // When the submitted layout picks a different active column, niri must not
    // auto-scroll the workspace to fit it — the scroll position should be the
    // same as it was before the apply.
    let mut layout = make_layout_with_output();
    add_tiling(&mut layout, 1);
    add_tiling(&mut layout, 2);
    add_tiling(&mut layout, 3);
    // Focus the leftmost column so view_pos is at the workspace origin.
    Op::FocusColumnLeft.apply(&mut layout);
    Op::FocusColumnLeft.apply(&mut layout);

    let before = view_pos(&layout);

    let mut tree = get_tree(&layout);
    tree.active_column_idx = 2;
    apply_tree(&mut layout, tree);

    let after = view_pos(&layout);
    assert!(
        (before - after).abs() < 1e-6,
        "view_pos changed when only the active column was reassigned: {before} -> {after}"
    );
}
