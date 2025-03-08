use std::{collections::HashMap, sync::Arc};
// use std::iter::FusedIterator;

use arc_swap::ArcSwap;
use poise::serenity_prelude::ChannelId;
use tokio::sync::RwLock;

use super::GameData;

#[derive(Clone, Debug, Default)]
pub struct GamesCache(Arc<RwLock<HashMap<ChannelId, Arc<ArcSwap<GameData>>>>>);

impl GamesCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn get(&self, channel_id: ChannelId) -> Option<Arc<GameData>> {
        let guard = self.0.read().await;
        guard.get(&channel_id).map(|arc_swap| arc_swap.load_full())
    }

    pub async fn _channel_is_locked(&self, id: ChannelId) -> bool {
        self.get(id).await.is_some()
    }

    pub async fn set(&self, channel_id: ChannelId, new_data: GameData) -> Arc<GameData> {
        let arc = Arc::new(new_data);
        let mut guard = self.0.write().await;
        if let Some(arc_swap) = guard.get_mut(&channel_id) {
            arc_swap.store(arc.clone());
            arc
        } else {
            guard.insert(channel_id, Arc::new(ArcSwap::new(arc.clone())));
            arc
        }
    }

    pub async fn remove(&self, channel_id: ChannelId) {
        let mut guard = self.0.write().await;
        guard.remove(&channel_id);
    }

    pub async fn unlock_channel(&self, id: ChannelId) {
        self.remove(id).await;
    }

    /*     pub async fn iter(&self) -> Iter {
        let vec: Vec<Arc<GameData>> = self
            .0
            .read()
            .await
            .values()
            .map(|arc_swap| arc_swap.load_full())
            .collect();

        Iter::new(vec)
    } */
}

/* #[derive(Default, Clone, Debug)]
pub struct Iter {
    vec: Vec<Arc<GameData>>,
    next_index: usize,
}

impl Iter {
    fn new(vec: impl Into<Vec<Arc<GameData>>>) -> Self {
        Self {
            vec: vec.into(),
            ..Self::default()
        }
    }
}

impl Iterator for Iter {
    type Item = Arc<GameData>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_index += 1;
        self.vec.get(self.next_index - 1).cloned()
    }
}

impl ExactSizeIterator for Iter {
    fn len(&self) -> usize {
        self.vec.len()
    }
}

impl FusedIterator for Iter {}
 */
