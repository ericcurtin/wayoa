//! wl_data_device protocol implementation
//!
//! Implements clipboard and drag-and-drop functionality.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use log::debug;

use crate::compositor::SurfaceId;

/// Unique identifier for data sources
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataSourceId(pub u64);

impl DataSourceId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        DataSourceId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Unique identifier for data offers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DataOfferId(pub u64);

impl DataOfferId {
    fn new() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        DataOfferId(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// A data source (clipboard or drag source)
#[derive(Debug)]
pub struct DataSource {
    /// Unique identifier
    pub id: DataSourceId,
    /// MIME types offered by this source
    pub mime_types: Vec<String>,
    /// Supported DnD actions
    pub dnd_actions: DndActions,
}

impl DataSource {
    /// Create a new data source
    pub fn new() -> Self {
        Self {
            id: DataSourceId::new(),
            mime_types: Vec::new(),
            dnd_actions: DndActions::empty(),
        }
    }

    /// Add a MIME type
    pub fn offer(&mut self, mime_type: String) {
        if !self.mime_types.contains(&mime_type) {
            self.mime_types.push(mime_type);
        }
    }

    /// Set DnD actions
    pub fn set_actions(&mut self, actions: DndActions) {
        self.dnd_actions = actions;
    }
}

impl Default for DataSource {
    fn default() -> Self {
        Self::new()
    }
}

/// A data offer (clipboard or drag offer to receiver)
#[derive(Debug)]
pub struct DataOffer {
    /// Unique identifier
    pub id: DataOfferId,
    /// Source this offer represents
    pub source_id: DataSourceId,
    /// MIME types available
    pub mime_types: Vec<String>,
    /// Source DnD actions
    pub source_actions: DndActions,
    /// Preferred action chosen by receiver
    pub preferred_action: DndAction,
    /// Final negotiated action
    pub action: DndAction,
}

impl DataOffer {
    /// Create a new data offer from a source
    pub fn new(source: &DataSource) -> Self {
        Self {
            id: DataOfferId::new(),
            source_id: source.id,
            mime_types: source.mime_types.clone(),
            source_actions: source.dnd_actions,
            preferred_action: DndAction::None,
            action: DndAction::None,
        }
    }

    /// Accept a MIME type
    pub fn accept(&mut self, _serial: u32, _mime_type: Option<String>) {
        // Client accepts this MIME type for DnD
    }

    /// Set preferred action
    pub fn set_actions(&mut self, actions: DndActions, preferred: DndAction) {
        self.preferred_action = preferred;
        // Negotiate action
        let available = self.source_actions.intersection(actions);
        self.action = if available.contains(DndActions::COPY) && preferred == DndAction::Copy {
            DndAction::Copy
        } else if available.contains(DndActions::MOVE) && preferred == DndAction::Move {
            DndAction::Move
        } else if available.contains(DndActions::ASK) && preferred == DndAction::Ask {
            DndAction::Ask
        } else if available.contains(DndActions::COPY) {
            DndAction::Copy
        } else if available.contains(DndActions::MOVE) {
            DndAction::Move
        } else {
            DndAction::None
        };
    }

    /// Finish the drag (data received)
    pub fn finish(&self) {
        debug!("Data offer finished");
    }
}

bitflags::bitflags! {
    /// DnD action flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DndActions: u32 {
        const NONE = 0;
        const COPY = 1;
        const MOVE = 2;
        const ASK = 4;
    }
}

/// Single DnD action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DndAction {
    #[default]
    None,
    Copy,
    Move,
    Ask,
}

/// Handler for wl_data_device and related protocols
pub struct DataDeviceHandler {
    sources: HashMap<DataSourceId, DataSource>,
    offers: HashMap<DataOfferId, DataOffer>,
    /// Current clipboard selection source
    selection: Option<DataSourceId>,
    /// Current DnD source
    dnd_source: Option<DataSourceId>,
    /// Surface being dragged over (will be used for full DnD implementation)
    #[allow(dead_code)]
    dnd_focus: Option<SurfaceId>,
}

impl DataDeviceHandler {
    /// Create a new data device handler
    pub fn new() -> Self {
        Self {
            sources: HashMap::new(),
            offers: HashMap::new(),
            selection: None,
            dnd_source: None,
            dnd_focus: None,
        }
    }

    /// Create a new data source
    pub fn create_data_source(&mut self) -> DataSourceId {
        let source = DataSource::new();
        let id = source.id;
        self.sources.insert(id, source);
        debug!("Created data source {:?}", id);
        id
    }

    /// Get a data source
    pub fn get_source(&self, id: DataSourceId) -> Option<&DataSource> {
        self.sources.get(&id)
    }

    /// Get a mutable data source
    pub fn get_source_mut(&mut self, id: DataSourceId) -> Option<&mut DataSource> {
        self.sources.get_mut(&id)
    }

    /// Destroy a data source
    pub fn destroy_source(&mut self, id: DataSourceId) {
        self.sources.remove(&id);
        if self.selection == Some(id) {
            self.selection = None;
        }
        if self.dnd_source == Some(id) {
            self.dnd_source = None;
        }
    }

    /// Set the clipboard selection
    pub fn set_selection(&mut self, source_id: Option<DataSourceId>, _serial: u32) {
        self.selection = source_id;
        debug!("Selection set to {:?}", source_id);
    }

    /// Get the current selection
    pub fn selection(&self) -> Option<&DataSource> {
        self.selection.and_then(|id| self.sources.get(&id))
    }

    /// Start a drag operation
    pub fn start_drag(
        &mut self,
        source_id: Option<DataSourceId>,
        _origin: SurfaceId,
        _icon: Option<SurfaceId>,
        _serial: u32,
    ) {
        self.dnd_source = source_id;
        debug!("Started drag with source {:?}", source_id);
    }

    /// Create an offer for a surface
    pub fn create_offer(&mut self, source_id: DataSourceId) -> Option<DataOfferId> {
        let source = self.sources.get(&source_id)?;
        let offer = DataOffer::new(source);
        let id = offer.id;
        self.offers.insert(id, offer);
        Some(id)
    }

    /// Get an offer
    pub fn get_offer(&self, id: DataOfferId) -> Option<&DataOffer> {
        self.offers.get(&id)
    }

    /// Get a mutable offer
    pub fn get_offer_mut(&mut self, id: DataOfferId) -> Option<&mut DataOffer> {
        self.offers.get_mut(&id)
    }

    /// Destroy an offer
    pub fn destroy_offer(&mut self, id: DataOfferId) {
        self.offers.remove(&id);
    }
}

impl Default for DataDeviceHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_source() {
        let mut source = DataSource::new();
        source.offer("text/plain".to_string());
        source.offer("text/html".to_string());
        assert_eq!(source.mime_types.len(), 2);
    }

    #[test]
    fn test_data_device_handler() {
        let mut handler = DataDeviceHandler::new();

        let source_id = handler.create_data_source();
        handler
            .get_source_mut(source_id)
            .unwrap()
            .offer("text/plain".to_string());

        handler.set_selection(Some(source_id), 1);
        assert!(handler.selection().is_some());

        let offer_id = handler.create_offer(source_id).unwrap();
        assert!(handler.get_offer(offer_id).is_some());
    }

    #[test]
    fn test_dnd_action_negotiation() {
        let mut source = DataSource::new();
        source.set_actions(DndActions::COPY | DndActions::MOVE);

        let mut offer = DataOffer::new(&source);
        offer.set_actions(DndActions::COPY, DndAction::Copy);
        assert_eq!(offer.action, DndAction::Copy);
    }
}
