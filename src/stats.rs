use std::cmp::Ord;
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant};

use axum::response::IntoResponse;
use dashmap::DashMap;
use serde::Serialize;

use crate::token::Token;
use crate::user::User;

/// Max number of tokens that the server will keep in memory.
const MAX_USER_TOKENS_CACHE: usize = 100_000;

pub struct StatsCollectorUser {
    pub best_1m: usize,
    pub requests_1m: VecDeque<Instant>,
    validated_tokens_queue: VecDeque<Token>,
    validated_tokens_set: HashSet<Token>,
}

impl Default for StatsCollectorUser {
    fn default() -> Self {
        Self {
            best_1m: 0,
            requests_1m: VecDeque::new(),
            validated_tokens_queue: VecDeque::with_capacity(MAX_USER_TOKENS_CACHE),
            validated_tokens_set: HashSet::with_capacity(MAX_USER_TOKENS_CACHE),
        }
    }
}

impl StatsCollectorUser {
    fn garbage_collect(&mut self, now: Instant) {
        while self
            .requests_1m
            .front()
            .map(|req| now - *req > Duration::from_secs(60))
            .unwrap_or(false)
        {
            self.requests_1m.pop_front();
        }
    }

    fn update_best(&mut self) {
        let now = Instant::now();
        self.garbage_collect(now);
        let curr_1m = self.requests_1m.len();

        if curr_1m > self.best_1m {
            self.best_1m = curr_1m;
        }
    }

    pub fn made_request(&mut self) {
        let now = Instant::now();
        self.garbage_collect(now);
        self.requests_1m.push_back(now);
        self.update_best();
    }
}

#[derive(Serialize)]
pub struct LadderItem {
    user: User,
    perf_1m: usize,
    best_1m: usize,
}

#[derive(Default)]
pub struct StatsCollector {
    user_stats: DashMap<User, StatsCollectorUser>,
}

impl StatsCollector {
    pub fn made_request(&self, user: User, token: Token) -> bool {
        let mut user_stats = self.user_stats.entry(user).or_default();

        if user_stats.validated_tokens_set.contains(&token) {
            return true;
        }

        if MAX_USER_TOKENS_CACHE > 0 {
            while user_stats.validated_tokens_set.len() >= MAX_USER_TOKENS_CACHE {
                let popped_token = user_stats
                    .validated_tokens_queue
                    .pop_front()
                    .expect("MAX_USER_TOKEN_CACHE is positive");

                user_stats.validated_tokens_set.remove(&popped_token);
            }

            user_stats.validated_tokens_queue.push_back(token);
            user_stats.validated_tokens_set.insert(token);
        }

        user_stats.made_request();
        false
    }

    pub fn ladder(&self) -> Vec<LadderItem> {
        let mut res: Vec<_> = self
            .user_stats
            .iter_mut()
            .map(|mut item| {
                item.update_best();

                LadderItem {
                    user: item.key().clone(),
                    perf_1m: item.requests_1m.len(),
                    best_1m: item.best_1m,
                }
            })
            .collect();

        res.sort_by(|x, y| {
            (x.best_1m.cmp(&y.best_1m))
                .then(x.perf_1m.cmp(&y.perf_1m))
                .reverse()
        });

        res
    }
}
