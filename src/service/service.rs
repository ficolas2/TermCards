use crate::repository::repository::Repository;

pub struct Service {
    pub(in crate::service) repository: Repository,
}

impl Service {
    pub fn new(repository: Repository) -> Service {
        Service { repository }
    }
}
