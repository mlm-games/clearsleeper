use clearsleeper::sim::sea::{GRID_CELLS, GRID_SIZE, Sea, SeaTables};

#[test]
fn golden_sea_calm_grid_fades_at_edges() {
    let mut sea = Sea::new(SeaTables::new(|_| 0));
    sea.execute(40_000.0, 0.0);
    assert!(!sea.culled());
    assert_eq!(sea.flat(), 0.0);

    let heights = sea.heights();
    assert_eq!(heights[32 * GRID_CELLS + 32], 1.0);
    assert_eq!(heights[32 * GRID_CELLS + 1], 1.0 / 6.0);
    assert_eq!(heights[1 * GRID_CELLS + 32], 1.0 / 6.0);
    assert_eq!(heights[1 * GRID_CELLS + 1], (1.0 / 6.0) * (1.0 / 6.0));
    assert_eq!(heights[63 * GRID_CELLS + 1], (1.0 / 6.0) * (1.0 / 6.0));
    assert_eq!(heights[1 * GRID_CELLS + 63], (1.0 / 6.0) * (1.0 / 6.0));
    assert_eq!(
        heights[59 * GRID_CELLS + 1],
        heights[1 * GRID_CELLS + 5],
        "mirrored fade rows must share weights"
    );
    assert_eq!(heights[0], 1.0);
    assert_eq!(heights[GRID_CELLS - 1], 1.0);
    assert_eq!(heights[(GRID_CELLS - 1) * GRID_CELLS], 1.0);
    assert_eq!(heights[GRID_CELLS * GRID_CELLS - 1], 1.0);
}

#[test]
fn golden_sea_flat_inter_fades_from_calm_shore() {
    let mut sea = Sea::new(SeaTables::new(|room| if room == 25 { 0 } else { 10 }));

    sea.execute(0.0, 0.0);
    assert_eq!(sea.flat(), 0.0);
    assert!(sea.culled());

    sea.execute(70_000.0, 0.0);
    assert!(!sea.culled());
    assert_eq!(sea.flat(), 7_200.0 / 12_800.0);

    sea.execute(140_000.0, 0.0);
    assert_eq!(sea.flat(), 1.0);
}

#[test]
fn golden_sea_cull_freezes_the_grid() {
    let mut sea = Sea::new(SeaTables::new(|room| if room == 25 { 0 } else { 10 }));

    sea.execute(140_000.0, 0.0);
    assert!(!sea.culled());
    let frozen = sea.heights().to_vec();

    sea.execute(0.0, 0.0);
    assert!(sea.culled());
    assert_eq!(sea.origin(), glam::Vec2::new(-25_600.0, -25_600.0));
    assert_eq!(sea.heights(), frozen.as_slice());
    assert_eq!(sea.height_at(0.0, 0.0), 1.0);

    sea.execute(140_000.0, 0.0);
    assert_ne!(
        sea.heights(),
        frozen.as_slice(),
        "waves must resume outside the calm cell"
    );
}

#[test]
fn golden_sea_counter_ratios_wrap_per_wave() {
    let mut sea = Sea::new(SeaTables::new(|_| 10));

    sea.execute(140_000.0, 0.0);
    assert_eq!(sea.counter_ratio(0), 1.0 / 200.0);
    assert_eq!(sea.counter_ratio(1), 1.0 / 190.0);
    assert_eq!(sea.counter_ratio(2), 1.0 / 210.0);
    assert_eq!(sea.counter_ratio(3), 1.0 / 180.0);

    for _ in 0..179 {
        sea.execute(140_000.0, 0.0);
    }
    assert_eq!(sea.counter_ratio(3), 0.0, "wave 3 wraps at 180");
    assert_eq!(sea.counter_ratio(0), 180.0 / 200.0);
}

#[test]
fn golden_sea_wave_samples_follow_grid_planes() {
    let mut sea = Sea::new(SeaTables::new(|_| 0));
    sea.execute(40_000.0, 0.0);

    let origin_x = 40_000.0 - 25_600.0;
    let origin_z = 0.0 - 25_600.0;
    assert_eq!(sea.height_at(origin_x - 1.0, 0.0), 1.0);

    let sample_x = origin_x + 3.5 * GRID_SIZE;
    let sample_z = origin_z + 32.5 * GRID_SIZE;
    let height = sea.height_at(sample_x, sample_z);
    assert!(
        (height - 0.583_333_3).abs() < 1e-4,
        "plane sample {height}"
    );
}

#[test]
fn golden_sea_open_water_waves_advance_each_tick() {
    let mut sea = Sea::new(SeaTables::new(|_| 10));

    sea.execute(140_000.0, 0.0);
    let before = sea.heights()[32 * GRID_CELLS + 32];
    let scale_before = peak(&sea);

    sea.execute(140_000.0, 0.0);
    let after = sea.heights()[32 * GRID_CELLS + 32];
    assert!((before - after).abs() > 1e-6, "waves must advance: {before}");

    for _ in 0..300 {
        sea.execute(140_000.0, 0.0);
    }
    let scale_after = peak(&sea);
    assert!(
        scale_after > scale_before * 1.5,
        "wave scale must grow: {scale_before} -> {scale_after}"
    );
}

fn peak(sea: &Sea) -> f32 {
    let heights = sea.heights();
    let mut peak = 0.0_f32;
    for z in 1..GRID_CELLS - 1 {
        for x in 1..GRID_CELLS - 1 {
            peak = peak.max((heights[z * GRID_CELLS + x] - 1.0).abs());
        }
    }
    peak
}
