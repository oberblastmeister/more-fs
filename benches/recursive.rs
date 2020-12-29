use std::{path::Path, process::Command};

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};

use test_dir::{fs_fn, join_all, TestDir};

pub fn clone_repo<P: AsRef<Path>>(url: &str, path: P) {
    let path = path.as_ref();

    println!("git cloning path {}", path.display());
    let status = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(&path)
        .status()
        .expect("Failed to get status");
    println!("Exit status {}", status);
}

fs_fn! {
    fn criterion_benchmark(c: &mut Criterion)(dir) {
        let (from, to) = join_all!(dir, "rust_from", "rust_to");
        clone_repo("https://github.com/rust-lang/rust.git", &from);

        let mut group = c.benchmark_group("recursive copy");

        let setup = || {
            if to.exists() {
                more_fs::remove_dir_all(&to).unwrap();
            }
        };

        group.bench_function("single threaded copy more_fs", |b| b.iter_batched(
                setup,
                |_| more_fs::copy_dir_all(&from, &to).unwrap(), BatchSize::PerIteration
        ));

        group.bench_function("multi threaded copy more_fs", |b| b.iter_batched(
                setup,
                |_| more_fs::copy_dir_all_par(&from, &to).unwrap(), BatchSize::PerIteration
        ));

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(50);
    targets = criterion_benchmark
}

criterion_main!(benches);
