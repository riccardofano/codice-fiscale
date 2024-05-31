use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng};

use codice_fiscale::{CodiceFiscale, Gender, NaiveDate, Subject};

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));
const GENDERS: [Gender; 2] = [Gender::Male, Gender::Female];
const ALLOWED_CHARS: [char; 27] = [
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', ' ',
];

fn random_name(rng: &mut ThreadRng) -> String {
    let length = rng.gen_range(1..20);
    ALLOWED_CHARS.choose_multiple(rng, length).collect()
}

fn random_place(rng: &mut ThreadRng) -> (String, String) {
    let pool = if rng.gen_bool(0.5) {
        &ACTIVE_PLACES
    } else {
        &INACTIVE_PLACES
    };
    let place = pool.keys().choose(rng).unwrap();
    place
        .split_once(',')
        .map(|(c, p)| (c.into(), p.into()))
        .unwrap()
}

fn random_date(rng: &mut ThreadRng) -> NaiveDate {
    NaiveDate::from_ymd_opt(
        rng.gen_range(1700..2100),
        rng.gen_range(1..=12),
        // Don't want to bother with invalid month days
        rng.gen_range(1..=28),
    )
    .unwrap()
}

fn create_random_subject() -> Subject {
    let mut rng = thread_rng();
    let (city, province) = random_place(&mut rng);

    Subject {
        first_name: random_name(&mut rng),
        last_name: random_name(&mut rng),
        birth_date: random_date(&mut rng),
        gender: *GENDERS.choose(&mut rng).unwrap(),
        birth_place: city,
        birth_province: province,
    }
}

fn bench(c: &mut Criterion) {
    c.bench_function("random subjects", |b| {
        b.iter_batched(
            create_random_subject,
            |subject| CodiceFiscale::try_from(&subject),
            BatchSize::SmallInput,
        )
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);