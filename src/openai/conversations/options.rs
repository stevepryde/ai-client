use crate::openai::responses::{
    ListOrder, ResponseInclude, ResponseItemId, ResponseOperationError,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConversationItemIncludeOptions {
    include: Vec<ResponseInclude>,
}

impl ConversationItemIncludeOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.include.push(include);
        self
    }

    pub fn includes(&self) -> &[ResponseInclude] {
        &self.include
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        include_query(&self.include)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ListConversationItemsOptions {
    limit: Option<u8>,
    order: Option<ListOrder>,
    after: Option<ResponseItemId>,
    include: Vec<ResponseInclude>,
}

impl ListConversationItemsOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn limit(mut self, limit: u8) -> Result<Self, ResponseOperationError> {
        if !(1..=100).contains(&limit) {
            return Err(ResponseOperationError::InvalidPageLimit);
        }
        self.limit = Some(limit);
        Ok(self)
    }

    pub fn order(mut self, order: ListOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn after(mut self, after: ResponseItemId) -> Self {
        self.after = Some(after);
        self
    }

    pub fn include(mut self, include: ResponseInclude) -> Self {
        self.include.push(include);
        self
    }

    pub fn page_limit(&self) -> u8 {
        self.limit.unwrap_or(20)
    }

    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut query = Vec::new();
        if let Some(limit) = self.limit {
            query.push(("limit".into(), limit.to_string()));
        }
        if let Some(order) = self.order {
            query.push(("order".into(), order.as_str().into()));
        }
        if let Some(after) = &self.after {
            query.push(("after".into(), after.as_str().into()));
        }
        query.extend(include_query(&self.include));
        query
    }
}

fn include_query(include: &[ResponseInclude]) -> Vec<(String, String)> {
    include
        .iter()
        .map(|include| ("include".into(), include.as_str().into()))
        .collect()
}
