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

#[derive(Debug)]
pub struct Token<'u> {
    index: u8,
    source: u64,
    user: &'u User,
    nb_children: u8,
    compute_time: Duration,
}

impl<'u> Token<'u> {
    pub fn new(index: u8, source: u64, user: &'u User) -> Self {
        let mut rng = SmallRng::seed_from_u64(source);

        let compute_time_sqrt = ((rng.sample::<f32, _>(StandardNormal) + 3.0) / 6.0)
            .clamp(MIN_COMPUTE_TIME, MAX_COMPUTE_TIME);

        Self {
            index,
            source,
            user,
            nb_children: rng.random_range(1..MAX_CHILDREN),
            compute_time: Duration::from_secs_f32(compute_time_sqrt.powi(2)),
        }
    }

    pub fn target(&self) -> u64 {
        let mut hash = seahash::SeaHasher::default();
        hash.write(SUPER_SECRET.as_bytes());
        hash.write(self.user.id.as_bytes());
        hash.write_u8(self.index);
        hash.write_u64(self.source);
        seahash::hash(&hash.finish().to_le_bytes())
    }

    pub fn from_user(user: &'u User) -> Self {
        let source = seahash::hash(user.id.as_bytes());
        Self::new(0, source, user)
    }

    pub fn validate_from_hex(user: &'u User, val: &[u8]) -> Option<Self> {
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
        let source = self.target();
        (0..self.nb_children).map(move |index| Token::new(index, source, self.user))
    }

    pub async fn compute(&self) {
        tracing::trace!("Compute for {:?}", self.compute_time);
        tokio::time::sleep(self.compute_time).await
    }
}
