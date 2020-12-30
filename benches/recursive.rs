use std::{path::Path, process::Command, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkGroup, Criterion};

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

fn bench_on_repo<M: criterion::measurement::Measurement>(
    dir: &TestDir,
    url: &str,
    from: &str,
    to: &str,
    group: &mut BenchmarkGroup<'_, M>,
) {
    join_all!(dir, from, to);

    clone_repo(url, &from);

    let setup = || {
        if to.exists() {
            more_fs::remove_dir_all(&to).unwrap();
        }
    };

    group.bench_function(format!("single threaded more_fs {}", url), |b| {
        b.iter_batched(
            setup,
            |_| more_fs::copy_dir_all(&from, &to).unwrap(),
            BatchSize::PerIteration,
        )
    });

    #[cfg(feature = "rayon")]
    group.bench_function(format!("multi threaded more_fs {}", url), |b| {
        b.iter_batched(
            setup,
            |_| more_fs::copy_dir_all_par(&from, &to).unwrap(),
            BatchSize::PerIteration,
        )
    });

    let mut fs_extra_copy_opt = fs_extra::dir::CopyOptions::new();
    fs_extra_copy_opt.copy_inside = true;

    group.bench_function(format!("single threaded fs_extra {}", url), |b| {
        b.iter_batched(
            setup,
            |_| fs_extra::dir::copy(&from, &to, &fs_extra_copy_opt).unwrap(),
            BatchSize::PerIteration,
        )
    });
}

fs_fn! {
    fn copy_benchmark(c: &mut Criterion)(dir) {
        let mut group = c.benchmark_group("recursive copy functions");

        bench_on_repo(&dir,
            "https://github.com/rust-lang/rust.git",
            "rust_from",
            "rust_to",
            &mut group
        );

        bench_on_repo(&dir,
            "https://github.com/sharkdp/fd.git",
            "fd_from",
            "fd_to",
            &mut group
        );

        bench_on_repo(&dir,
            "https://github.com/oberblastmeister/more-fs.git",
            "more_fs_from",
            "more_fs_to",
            &mut group
        );

        group.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .measurement_time(Duration::from_secs(10));
    targets = copy_benchmark
}

criterion_main!(benches);
