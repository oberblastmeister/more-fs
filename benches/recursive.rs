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

fn bench_on_repo(
    c: &mut Criterion,
    dir: &TestDir,
    url: &str,
    from: &str,
    to: &str,
    group_name: &str,
) {
    join_all!(dir, from, to);

    clone_repo(url, &from);

    let mut group = c.benchmark_group(group_name);

    let setup = || {
        if to.exists() {
            more_fs::remove_dir_all(&to).unwrap();
        }
    };

    group.bench_function("single threaded more_fs", |b| {
        b.iter_batched(
            setup,
            |_| more_fs::copy_dir_all(&from, &to).unwrap(),
            BatchSize::PerIteration,
        )
    });

    group.bench_function("multi threaded more_fs", |b| {
        b.iter_batched(
            setup,
            |_| more_fs::copy_dir_all_par(&from, &to).unwrap(),
            BatchSize::PerIteration,
        )
    });

    let mut fs_extra_copy_opt = fs_extra::dir::CopyOptions::new();
    fs_extra_copy_opt.copy_inside = true;

    group.bench_function("single threaded fs_extra", |b| {
        b.iter_batched(
            setup,
            |_| fs_extra::dir::copy(&from, &to, &fs_extra_copy_opt).unwrap(),
            BatchSize::PerIteration,
        )
    });

    group.finish();
}

fs_fn! {
    fn rust_src_bench(c: &mut Criterion)(dir) {
        bench_on_repo(c, &dir, "https://github.com/rust-lang/rust.git", "rust_src_from", "rust_src_to", "recursive copy rust source code");
    }
}

fs_fn! {
    fn fd_src_bench(c: &mut Criterion)(dir) {
        bench_on_repo(c, &dir, "https://github.com/sharkdp/fd.git", "fd_from", "fd_to", "recursive copy fd source code (smaller repo)");
    }
}

fs_fn! {
    fn this_crate_src_bench(c: &mut Criterion)(dir) {
        bench_on_repo(c, &dir, "https://github.com/oberblastmeister/more-fs.git", "more_fs_from", "more_fs_to", "recursive copy more_fs source code (smallest repo)");
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(30);
    targets = rust_src_bench, fd_src_bench, this_crate_src_bench
}

criterion_main!(benches);
