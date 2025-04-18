use std::hash::Hasher;
use std::time::Duration;

use rand::rngs::SmallRng;
use rand::{Rng, SeedableRng};
use rand_distr::StandardNormal;

use crate::user::User;

const SUPER_SECRET: &str = "<SUPER SECRET!>";
const MAX_CHILDREN: u8 = 8;
const MIN_COMPUTE_TIME: f32 = 0.005;
const MAX_COMPUTE_TIME: f32 = 1.0;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct Token {
    index: u8,
    source: u64,
    user_hash: u64,
}

impl Token {
    pub fn new(index: u8, source: u64, user: &User) -> Self {
        Self {
            index,
            source,
            user_hash: seahash::hash(user.id.as_bytes()),
        }
    }

    pub fn target(&self) -> u64 {
        let mut hash = seahash::SeaHasher::default();
        hash.write(SUPER_SECRET.as_bytes());
        hash.write_u64(self.user_hash);
        hash.write_u8(self.index);
        hash.write_u64(self.source);
        seahash::hash(&hash.finish().to_le_bytes())
    }

    pub fn from_user(user: &User) -> Self {
        let mut rng = rand::rng();
        Self::new(0, rng.random(), user)
    }

    pub fn validate_from_hex(user: &User, val: &[u8]) -> Option<Self> {
        let mut bytes = [0; 17];
        hex::decode_to_slice(val, &mut bytes).ok()?;
        let index = u8::from_le_bytes([bytes[0]]);
        let source = u64::from_le_bytes(bytes[1..9].try_into().unwrap());
        let target = u64::from_le_bytes(bytes[9..].try_into().unwrap());
        let token = Self::new(index, source, user);

        if target != token.target() {
            return None;
        }

        Some(token)
    }

    pub fn as_hex(&self) -> String {
        hex::encode(
            [
                &self.index.to_le_bytes() as &[_],
                &self.source.to_le_bytes(),
                &self.target().to_le_bytes(),
            ]
            .concat(),
        )
    }

    pub fn iter_children(&self) -> impl Iterator<Item = Token> {
        let mut rng = SmallRng::seed_from_u64(self.source);
        let nb_children = rng.random_range(1..=MAX_CHILDREN);
        let source = self.target();

        (0..nb_children).map(move |index| Token {
            index,
            source,
            user_hash: self.user_hash,
        })
    }

    pub async fn compute(&self) {
        let mut rng = SmallRng::seed_from_u64(self.source);

        let compute_time_sqrt = ((rng.sample::<f32, _>(StandardNormal) + 3.0) / 6.0)
            .clamp(MIN_COMPUTE_TIME, MAX_COMPUTE_TIME);

        let compute_time = Duration::from_secs_f32(compute_time_sqrt.powi(2));
        tracing::trace!("Compute for {compute_time:?}");
        tokio::time::sleep(compute_time).await
    }
}
