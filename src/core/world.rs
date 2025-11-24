use slotmap::{SecondaryMap, SlotMap, new_key_type};
use std::collections::VecDeque;
use crate::core::entity::{WorldEntity, spatial_transform::SpatialTransform};

new_key_type! {
    pub struct WorldEntityId;
}

/// Mostly just maintains the de facto spatial state of "things" in the world,
/// for other systems to reference.
pub struct World {
    entities: SlotMap<WorldEntityId, WorldEntity>,
    root_entity: WorldEntityId
}

impl World {
    /// Create a new world.
    pub fn new() -> Self {
        let mut entities = SlotMap::with_key();
        let root_entity = entities.insert(WorldEntity::new(
            None,
            vec![],
            SpatialTransform::identity(),
        ));
        Self {
            entities,
            root_entity
        }
    }
    
    /// Add the given entity and return their ID.
    pub fn add_entity(&mut self, mut parent: Option<WorldEntityId>, children: Vec<WorldEntityId>, local_transform: SpatialTransform) -> WorldEntityId {
        if parent.is_none() {
            parent = Some(self.root_entity);
        }
        let entity = WorldEntity::new(
            parent, 
            children, 
            local_transform
        );
        self.entities.insert(entity)
    }

    /// Get the given entity.
    pub fn entity(&self, id: WorldEntityId) -> Option<&WorldEntity> {
        self.entities.get(id)
    }

    /// Get the given entity mutably.
    pub fn entity_mut(&mut self, id: WorldEntityId) -> Option<&mut WorldEntity> {
        self.entities.get_mut(id)
    }

    /// Walks the entity graph and propagates each entity's transforms to its children's parent transforms.
    fn update_graph(&mut self) {
        let mut node_queue = VecDeque::with_capacity(self.entities.len());
        node_queue.push_front(self.root_entity);
        while !node_queue.is_empty() {
            let cur_entity_id = node_queue.pop_back().unwrap();
            let cur_entity = self.entities.get_mut(cur_entity_id).unwrap();
            let children = cur_entity.children().clone();

            if !cur_entity.already_propagated() {
                cur_entity.set_already_propagated(true);
                let cur_parent = cur_entity.transform();
                for node in &children {
                    let node = self.entities.get_mut(*node).unwrap();
                    node.update_parent_transform(|parent| *parent = cur_parent);
                    node.set_already_propagated(false);
                }
            }

            for child in children {
                node_queue.push_front(child);
            }
        }
    }
}