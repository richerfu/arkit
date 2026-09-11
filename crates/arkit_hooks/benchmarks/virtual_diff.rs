//! Standalone host benchmark for the pure virtual-item diff.
//!
//! Run from the repository root:
//! `rustc --edition=2021 -O crates/arkit_hooks/benchmarks/virtual_diff.rs -o /tmp/arkit-virtual-diff-bench && /tmp/arkit-virtual-diff-bench`

use std::hint::black_box;
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "../src/virtual_diff.rs"]
mod virtual_diff;

use virtual_diff::{virtual_item_updates, VirtualItemStamp};

type Stamp = VirtualItemStamp<u32, u8>;

fn stamps(ids: impl IntoIterator<Item = u32>) -> Vec<Stamp> {
    ids.into_iter()
        .map(|id| VirtualItemStamp::new(id, 0))
        .collect()
}

struct BenchRng(u64);

impl BenchRng {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }

    fn shuffle<T>(&mut self, values: &mut [T]) {
        for index in (1..values.len()).rev() {
            values.swap(index, self.next() as usize % (index + 1));
        }
    }
}

fn cases(count: usize) -> Vec<(&'static str, Vec<Stamp>, Vec<Stamp>)> {
    let ordered = (0..count as u32).collect::<Vec<_>>();

    let mut reverse = ordered.clone();
    reverse.reverse();

    let mut rotate_left = ordered.clone();
    rotate_left.rotate_left((count / 7).max(1));

    let mut rotate_right = ordered.clone();
    rotate_right.rotate_right((count / 7).max(1));

    let mut rotate_left_one = ordered.clone();
    rotate_left_one.rotate_left(1);

    let mut rotate_right_one = ordered.clone();
    rotate_right_one.rotate_right(1);

    let mut random = ordered.clone();
    BenchRng(0x89a7_34cd_128f_ee51 ^ count as u64).shuffle(&mut random);

    let mut sparse = ordered.clone();
    for index in (0..count.saturating_sub(1)).step_by(101) {
        sparse.swap(index, index + 1);
    }

    let append = (0..(count + count / 10).max(1) as u32).collect::<Vec<_>>();
    let mut prepend = ((count as u32)..(count + count / 10).max(1) as u32).collect::<Vec<_>>();
    prepend.extend(ordered.iter().copied());

    let mut mixed_survivors = ordered
        .iter()
        .copied()
        .filter(|id| id % 11 != 0)
        .collect::<Vec<_>>();
    let mixed_len = mixed_survivors.len();
    if mixed_len > 1 {
        mixed_survivors.rotate_left((count / 97).max(1) % mixed_len);
    }
    let mut mixed = Vec::with_capacity(mixed_survivors.len() + count / 19 + 1);
    let mut fresh = count as u32;
    for (index, id) in mixed_survivors.into_iter().enumerate() {
        if index % 19 == 0 {
            mixed.push(fresh);
            fresh += 1;
        }
        mixed.push(id);
    }

    let unchanged = stamps(ordered.iter().copied());
    let mut revision = unchanged.clone();
    if let Some(stamp) = revision.get_mut(count / 2) {
        stamp.revision = 1;
    }

    vec![
        ("reverse", stamps(ordered.iter().copied()), stamps(reverse)),
        (
            "rotate-left",
            stamps(ordered.iter().copied()),
            stamps(rotate_left),
        ),
        (
            "rotate-right",
            stamps(ordered.iter().copied()),
            stamps(rotate_right),
        ),
        (
            "rotate-left-one",
            stamps(ordered.iter().copied()),
            stamps(rotate_left_one),
        ),
        (
            "rotate-right-one",
            stamps(ordered.iter().copied()),
            stamps(rotate_right_one),
        ),
        (
            "random-shuffle",
            stamps(ordered.iter().copied()),
            stamps(random),
        ),
        (
            "sparse-moves",
            stamps(ordered.iter().copied()),
            stamps(sparse),
        ),
        ("append", stamps(ordered.iter().copied()), stamps(append)),
        ("prepend", stamps(ordered.iter().copied()), stamps(prepend)),
        ("mixed", stamps(ordered), stamps(mixed)),
        ("unchanged", unchanged.clone(), unchanged.clone()),
        ("one-revision", unchanged, revision),
    ]
}

fn measure(previous: &[Stamp], next: &[Stamp], iterations: u32) -> (Duration, usize) {
    let start = Instant::now();
    let mut operation_count = 0;
    for _ in 0..iterations {
        let updates = virtual_item_updates(black_box(previous), black_box(next)).unwrap();
        operation_count = black_box(updates.len());
    }
    (start.elapsed() / iterations, operation_count)
}

fn main() {
    println!("count\tcase\taverage\toperations");
    for count in [1_000, 10_000, 100_000] {
        let iterations = match count {
            1_000 => 100,
            10_000 => 20,
            _ => 3,
        };
        for (name, previous, next) in cases(count) {
            let (elapsed, operations) = measure(&previous, &next, iterations);
            println!("{count}\t{name}\t{elapsed:?}\t{operations}");
        }
    }
}
