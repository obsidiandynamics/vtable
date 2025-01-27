use criterion::{criterion_group, criterion_main, Criterion};
use vtable::ched::{Token, CHED};
use std::any::Any;
use std::hash::{Hash, Hasher};
use vtable::every::BoxDowncast;

struct DummyHasher(bool);

impl Hasher for DummyHasher {
    fn finish(&self) -> u64 {
        if self.0 {
            1
        } else {
            0
        }
    }

    fn write(&mut self, _bytes: &[u8]) {
        self.0 = true;
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cri_box_new", |b| {
        b.iter(|| {
            let obj = Box::new(42) as Box<dyn Any>;
            obj
        });
    });

    c.bench_function("cri_box_clone", |b| {
        let obj = Box::new(42) as Box<dyn Any>;
        b.iter(|| {
            let val = obj.downcast_ref::<i32>().unwrap();
            let clone = Box::new(val.clone()) as Box<dyn Any>;
            clone
        });
    });

    c.bench_function("cri_box_eq", |b| {
        let obj_1 = Box::new(42) as Box<dyn Any>;
        let obj_2 = Box::new(42) as Box<dyn Any>;
        b.iter(|| {
            let val_1 = obj_1.downcast_ref::<i32>().unwrap();
            let val_2 = obj_2.downcast_ref::<i32>().unwrap();
            assert_eq!(val_1, val_2);
        });
    });

    c.bench_function("cri_dynamic_new", |b| {
        let tok = Token::default();
        b.iter(|| {
            let obj = CHED::new(42, &tok);
            obj
        });
    });

    c.bench_function("cri_dynamic_clone", |b| {
        let obj = CHED::new(42, &Token::default());
        b.iter(|| {
            let clone = obj.clone();
            clone
        });
    });

    c.bench_function("cri_dynamic_eq", |b| {
        let tok = Token::default();
        let obj_1 = CHED::new(42, &tok);
        let obj_2 = CHED::new(42, &tok);
        b.iter(|| {
            assert_eq!(obj_1, obj_2);
        });
    });

    c.bench_function("cri_dynamic_hash", |b| {
        let obj = CHED::new(42, &Token::default());
        let mut hasher = DummyHasher(false);
        b.iter(|| {
            obj.hash(&mut hasher);
            assert_eq!(1, hasher.finish());
        });
    });

    c.bench_function("cri_dynamic_downcast_ref", |b| {
        let obj = CHED::new(42i32, &Token::default());
        b.iter(|| {
            assert_eq!(Ok(&42i32), obj.inner().downcast_ref());
        });
    });

    c.bench_function("cri_dynamic_downcast", |b| {
        let tok = Token::default();
        b.iter_with_setup(
            || CHED::new(42i32, &tok),
            |obj| {
                assert_eq!(Ok(42i32), obj.into_inner().downcast());
            },
        );
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
