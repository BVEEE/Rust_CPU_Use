use criterion::{criterion_group, criterion_main, Criterion};
use sysinfo::{CpuExt, System, SystemExt};

fn bench_refresh_cpu(c: &mut Criterion) {
    let mut system = System::new_all();
    c.bench_function("refresh_cpu", |b| {
        b.iter(|| {
            system.refresh_cpu();
            let _ = system.global_cpu_info().cpu_usage();
        })
    });
}

fn bench_refresh_processes(c: &mut Criterion) {
    let mut system = System::new_all();
    c.bench_function("refresh_processes", |b| {
        b.iter(|| {
            system.refresh_processes();
            let _count = system.processes().len();
        })
    });
}

criterion_group!(cpu_benches, bench_refresh_cpu, bench_refresh_processes);
criterion_main!(cpu_benches);
