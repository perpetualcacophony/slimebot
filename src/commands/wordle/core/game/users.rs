use std::collections::HashMap;

use poise::serenity_prelude::{User, UserId};

use crate::utils::AsDiscordId;

pub enum Users<'owner> {
    One(&'owner User),
    More {
        owner: &'owner User,
        others: Option<UserMap>,
    },
}

impl<'owner> Users<'owner> {
    pub fn one(user: &'owner User) -> Self {
        Self::One(user)
    }

    pub fn more(owner: &'owner User) -> Self {
        Self::More {
            owner,
            others: None,
        }
    }

    pub fn count(&self) -> usize {
        match self {
            Self::One(..) => 1,
            Self::More { others, .. } => others.as_ref().map_or(0, |set| set.len()) + 1,
        }
    }

    pub fn owner(&self) -> &'owner User {
        match self {
            Self::One(owner) => owner,
            Self::More { owner, .. } => owner,
        }
    }

    pub fn user_map(&self) -> UserMapBorrowed {
        match self {
            Self::One(user) => [*user].into_iter().collect(),
            Self::More { owner, others } => {
                let mut new =
                    UserMapBorrowed::with_capacity(others.as_ref().map_or(1, |map| map.len()));
                new.insert(owner);
                new
            }
        }
    }

    pub fn contains(&self, user_id: &impl AsDiscordId<UserId>) -> bool {
        self.get(user_id).is_some()
    }

    fn get(&self, user_id: &impl AsDiscordId<UserId>) -> Option<&User> {
        let user_id = user_id.as_id();

        if user_id == self.owner().id {
            Some(self.owner())
        } else if let Self::More { others, .. } = self
            && let Some(others) = others
        {
            others.get(user_id)
        } else {
            None
        }
    }

    pub fn add(&mut self, user: User) {
        match self {
            Self::One(..) => (),
            Self::More { owner, others } => {
                if user != **owner {
                    let map = others.get_or_insert(UserMap::default());
                    map.insert(user);
                }
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct UserMap(HashMap<UserId, User>);

impl UserMap {
    fn new() -> Self {
        Self(HashMap::with_capacity(1))
    }

    fn iter(&self) -> std::collections::hash_map::Iter<'_, UserId, User> {
        self.0.iter()
    }

    fn insert(&mut self, user: User) {
        self.0.insert(user.id, user);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn get(&self, user_id: impl AsDiscordId<UserId>) -> Option<&User> {
        self.as_ref().get(&user_id.as_id())
    }

    fn contains(&self, user_id: impl AsDiscordId<UserId>) -> bool {
        self.get(user_id).is_some()
    }
}

impl From<HashMap<UserId, User>> for UserMap {
    fn from(value: HashMap<UserId, User>) -> Self {
        Self(value)
    }
}

impl FromIterator<User> for UserMap {
    fn from_iter<T: IntoIterator<Item = User>>(iter: T) -> Self {
        iter.into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>()
            .into()
    }
}

impl<'user> FromIterator<&'user User> for UserMap {
    fn from_iter<T: IntoIterator<Item = &'user User>>(iter: T) -> Self {
        iter.into_iter()
            .map(|user| (user.id, user.clone()))
            .collect::<HashMap<_, _>>()
            .into()
    }
}

impl AsRef<HashMap<UserId, User>> for UserMap {
    fn as_ref(&self) -> &HashMap<UserId, User> {
        &self.0
    }
}

impl<'map> IntoIterator for &'map UserMap {
    type Item = (&'map UserId, &'map User);
    type IntoIter = <&'map HashMap<UserId, User> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

impl From<UserMapBorrowed<'_>> for UserMap {
    fn from(value: UserMapBorrowed<'_>) -> Self {
        value.into_iter().map(|tup| *tup.1).collect()
    }
}

#[derive(Clone, Debug)]
struct UserMapBorrowed<'user>(HashMap<UserId, &'user User>);

impl<'user> UserMapBorrowed<'user> {
    fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, user: &'user User) {
        self.0.insert(user.id, user);
    }

    fn len(&self) -> usize {
        self.0.len()
    }

    fn with_capacity(capacity: usize) -> Self {
        Self(HashMap::with_capacity(capacity))
    }

    fn get(&self, user_id: impl AsDiscordId<UserId>) -> Option<&User> {
        self.as_ref().get(&user_id.as_id()).copied()
    }

    fn contains(&self, user_id: impl AsDiscordId<UserId>) -> bool {
        self.get(user_id).is_some()
    }
}

impl Default for UserMapBorrowed<'_> {
    fn default() -> Self {
        Self::with_capacity(1)
    }
}

impl<'user> From<HashMap<UserId, &'user User>> for UserMapBorrowed<'user> {
    fn from(value: HashMap<UserId, &'user User>) -> Self {
        Self(value)
    }
}

impl<'user> FromIterator<&'user User> for UserMapBorrowed<'user> {
    fn from_iter<T: IntoIterator<Item = &'user User>>(iter: T) -> Self {
        iter.into_iter()
            .map(|user| (user.id, user))
            .collect::<HashMap<_, _>>()
            .into()
    }
}

impl<'user> AsRef<HashMap<UserId, &'user User>> for UserMapBorrowed<'user> {
    fn as_ref(&self) -> &HashMap<UserId, &'user User> {
        &self.0
    }
}

impl<'map, 'user> IntoIterator for &'map UserMapBorrowed<'user> {
    type Item = (&'map UserId, &'map &'user User);
    type IntoIter = <&'map HashMap<UserId, &'user User> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.as_ref().iter()
    }
}

mod new {
    use std::{collections::HashMap, marker::PhantomData};

    use poise::serenity_prelude::{User, UserId};

    enum Owned {}
    enum Borrowed {}

    pub enum NewUserMap<'user, M> {
        Owned {
            map: HashMap<UserId, User>,
        },
        Borrowed {
            map: HashMap<UserId, &'user User>,
            phantom: PhantomData<M>,
        },
    }

    impl<M> NewUserMap<'_, M> {
        fn owned() -> Self {
            Self::Owned {
                map: HashMap::new(),
            }
        }

        fn borrowed() -> Self {
            Self::Borrowed {
                map: HashMap::new(),
                phantom: PhantomData,
            }
        }
    }

    impl NewUserMap<'_, Owned> {
        fn new() -> Self {
            Self::owned()
        }

        fn inner(&self) -> &HashMap<UserId, User> {
            match self {
                Self::Owned { map } => map,
                _ => unreachable!(),
            }
        }

        fn inner_mut(&mut self) -> &mut HashMap<UserId, User> {
            match self {
                Self::Owned { map } => map,
                _ => unreachable!(),
            }
        }

        fn insert(&mut self, user: User) {
            self.inner_mut().insert(user.id, user);
        }
    }

    impl NewUserMap<'_, Borrowed> {
        fn new() -> Self {
            Self::borrowed()
        }
    }
}
